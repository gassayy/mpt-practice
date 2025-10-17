# MPT Storage and Verification

## How Data is Stored in MPT

### Storage Architecture

Your MPT uses a **content-addressed storage** system:

```rust
// From trie.rs
pub struct MerklePatriciaTrie {
    storage: HashMap<Hash, Node>,  // Hash → Node mapping
    root: Hash,                     // 32-byte root hash
}
```

**Key Concept**: Nodes are stored by their **hash**, not by their position in the tree.

### Storage Process (Step by Step)

Let's trace inserting `"dog" → "puppy"`:

#### Step 1: Convert Key to Nibbles
```rust
let key = b"dog";  // [0x64, 0x6f, 0x67]
let nibbles = bytes_to_nibbles(key);
// Result: [6, 4, 6, 15, 6, 7]  (6 nibbles)
```

#### Step 2: Create Node with Raw Data
```rust
let node = Node::new_leaf(&nibbles, b"puppy".to_vec());
// Creates: Node::Leaf(
//     compact_encode([6,4,6,15,6,7]),  // Path
//     vec![112, 117, 112, 112, 121]    // Raw "puppy"
// )
```

#### Step 3: RLP Encode the Node
```rust
let encoded = node.encode_raw();
// RLP format: [list_marker, path_bytes, value_bytes]
// Example: [0xc8, 0x83, 0x64, 0x6f, 0x67, 0x85, 112, 117, 112, 112, 121]
```

#### Step 4: Hash the Encoded Node
```rust
let hash = keccak256(&encoded);
// Result: [0xab, 0xcd, ..., 0xef]  (32 bytes)
```

#### Step 5: Store Hash → Node Mapping
```rust
storage.insert(hash, node);
// Now: HashMap<0xabcd...ef, Node::Leaf(...)>

root = hash;  // Update root to point to this node
```

### Storage Example with Multiple Keys

Let's insert two related keys:

```
Insert "dog" → "puppy"
Insert "doge" → "coin"
```

**Resulting Tree Structure:**

```
Storage HashMap:
┌─────────────────────────────────────────────────┐
│ Hash₁ → Extension([6,4,6,15], Hash₂)           │  Root
│ Hash₂ → Branch([..., Hash₃, ...], None)        │  
│ Hash₃ → Extension([7], Hash₄)                  │
│ Hash₄ → Leaf([6,5], "coin")                    │  "doge"
│ Hash₅ → Leaf([], "puppy")                      │  "dog"
└─────────────────────────────────────────────────┘
     ↑
  root = Hash₁
```

**Visual Tree:**
```
                    Root (Hash₁)
                    Extension
                    path: [6,4,6,15] (nibbles for "do")
                    child: Hash₂
                        ↓
                    Branch (Hash₂)
                    ├─ [6]: Hash₃
                    └─ [7]: Hash₅
                       ↓           ↓
                  Extension     Leaf (Hash₅)
                  (Hash₃)       path: []
                  path: [7]     value: "puppy"
                  child: Hash₄
                      ↓
                  Leaf (Hash₄)
                  path: [6,5]
                  value: "coin"
```

### Why Content-Addressed Storage?

**Benefits:**

1. **Deduplication**: Same content = same hash = single storage
```rust
// Two identical nodes only stored once
let node1 = Node::new_leaf(&[1,2], b"value".to_vec());
let node2 = Node::new_leaf(&[1,2], b"value".to_vec());
// Both hash to same value → only one storage entry
```

2. **Integrity**: Can verify node hasn't been tampered with
```rust
let stored_node = storage.get(&hash).unwrap();
assert_eq!(stored_node.hash(), hash);  // Always true if not corrupted
```

3. **Historical Access**: Old roots still valid
```rust
let old_root = trie.root_hash();
trie.insert(b"new", b"data".to_vec());
// Can still access old_root's state if nodes are kept
```

---

## How to Verify a Key Exists in MPT

### Method 1: Direct Lookup (What Your Code Does Now)

```rust
// From your trie.rs
pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
    let nibbles = bytes_to_nibbles(key);
    self.get_at(&nibbles, self.root)
}
```

**Process:**
1. Convert key to nibbles
2. Start at root hash
3. Fetch node from storage
4. Follow the path through the tree
5. Return value if found, None otherwise

**Example:**
```rust
// To verify "doge" exists:
let nibbles = [6, 4, 6, 15, 6, 7, 6, 5];

1. Fetch root (Hash₁) → Extension node
   - path: [6,4,6,15] matches first 4 nibbles ✓
   - Continue with [6,7,6,5], follow child Hash₂

2. Fetch Hash₂ → Branch node
   - Next nibble is 6
   - Follow children[6] → Hash₃

3. Fetch Hash₃ → Extension node
   - path: [7] matches next nibble ✓
   - Continue with [6,5], follow child Hash₄

4. Fetch Hash₄ → Leaf node
   - path: [6,5] matches remaining nibbles ✓
   - Return value: "coin" ✓
```

**Complexity:** O(k × log₁₆(n)) where k = key length, n = number of keys

### Method 2: Merkle Proof (Light Client Verification)

**Problem**: What if you don't have the full tree?

**Solution**: A **Merkle proof** is a minimal set of nodes proving a key exists.

#### What is a Merkle Proof?

Instead of storing the entire trie, you only need:
- The root hash (trusted)
- The path of nodes from root to the key
- Sibling hashes at each level

**Example Proof for "doge":**

```
Proof = [
    Extension([6,4,6,15], Hash₂),     // Node at root
    Branch([..., Hash₃, ...], None),   // Node at Hash₂
    Extension([7], Hash₄),             // Node at Hash₃
    Leaf([6,5], "coin")                // Node at Hash₄
]

Root hash = 0xabcd...ef (trusted)
```

#### Verification Process

```
1. Start with trusted root hash
2. For each node in proof:
   a. RLP encode the node
   b. Hash it
   c. Verify hash matches expected
   d. Extract next reference from node
3. Final node should contain the value
```

**Trust Model:**
- You only trust the root hash (32 bytes)
- Verifier re-computes hashes upward
- If final hash matches root, proof is valid

---

## Comparison Table

| Method | Data Needed | Bandwidth | Trust | Use Case |
|--------|-------------|-----------|-------|----------|
| **Direct Lookup** | Full trie | N/A (local) | Full access | Full nodes |
| **Merkle Proof** | Proof nodes only | ~5-10 nodes | Root hash only | Light clients |

---

## Practical Example

### Current Implementation (Direct Lookup)

```rust
use mpt::MerklePatriciaTrie;

let mut trie = MerklePatriciaTrie::new();
trie.insert(b"dog", b"puppy".to_vec());
trie.insert(b"doge", b"coin".to_vec());

// Verify "doge" exists
match trie.get(b"doge") {
    Some(value) => println!("Found: {}", String::from_utf8_lossy(&value)),
    None => println!("Not found"),
}
// Output: Found: coin
```

### With Merkle Proofs (Not Yet Implemented)

```rust
// What it could look like:
let proof = trie.generate_proof(b"doge");
let root = trie.root_hash();

// On light client (doesn't have full trie):
let verified = verify_proof(b"doge", b"coin", &proof, root);
assert!(verified);  // True if proof is valid
```

---

## Storage Internals

### Memory Layout

```
MerklePatriciaTrie
├─ storage: HashMap<Hash, Node>
│  ├─ 0x1a2b... → Node::Extension(...)
│  ├─ 0x3c4d... → Node::Branch(...)
│  ├─ 0x5e6f... → Node::Leaf(...)
│  └─ ...
└─ root: Hash (0x1a2b...)
```

### What Gets Stored Where

```
User Data:
  Key: b"dog" → Inside node as compact-encoded nibbles
  Value: b"puppy" → Inside leaf node as raw bytes

Tree Structure:
  Node references → Hashed and used as HashMap keys
  Node contents → Stored as values in HashMap
```

### Storage Efficiency

For n keys with average length k:

- **Without compression**: ~k × n nodes
- **With extension nodes**: ~log₁₆(n) × shared_prefix_factor nodes
- **Hash overhead**: 32 bytes per node reference

---

## Next Steps: Implementing Merkle Proofs

Would you like me to implement proof generation and verification for your MPT?

This would add:
1. `generate_proof(key) → Vec<Node>` - Creates a proof
2. `verify_proof(key, value, proof, root_hash) → bool` - Verifies it

This is a common extension for course projects and demonstrates the full power of Merkle trees!

