# MPT Storage Visualization

## Complete Storage Example

This document visualizes exactly what happens when you insert keys into your MPT.

### Scenario: Insert "dog" and "doge"

```rust
let mut trie = MerklePatriciaTrie::new();
trie.insert(b"dog", b"puppy".to_vec());
trie.insert(b"doge", b"coin".to_vec());
```

---

## Step-by-Step Storage Process

### Step 1: Insert "dog" → "puppy"

**Key Conversion:**
```
"dog" = [0x64, 0x6f, 0x67]
       ↓ bytes_to_nibbles
     [6, 4, 6, 15, 6, 7]
```

**Node Created:**
```rust
Node::Leaf(
    compact_encode([6,4,6,15,6,7], true),  // Path with leaf flag
    vec![112, 117, 112, 112, 121]          // "puppy" as raw bytes
)
```

**Storage:**
```
┌─────────────────────────────────────────┐
│ HashMap<Hash, Node>                     │
├─────────────────────────────────────────┤
│ Hash₁ → Leaf([6,4,6,15,6,7], "puppy")  │
└─────────────────────────────────────────┘
     ↑
Root = Hash₁ = keccak256(RLP(leaf_node))
```

**Tree:**
```
Root (Hash₁)
  ↓
Leaf
├─ path: [6,4,6,15,6,7] (all of "dog")
└─ value: "puppy"
```

---

### Step 2: Insert "doge" → "coin"

**Key Conversion:**
```
"doge" = [0x64, 0x6f, 0x67, 0x65]
        ↓ bytes_to_nibbles
      [6, 4, 6, 15, 6, 7, 6, 5]
```

**Path Comparison:**
```
Existing: [6, 4, 6, 15, 6, 7]      ("dog")
New:      [6, 4, 6, 15, 6, 7, 6, 5] ("doge")
Common:   [6, 4, 6, 15, 6, 7]      (6 nibbles shared)
```

**Tree Restructuring:**

The single leaf splits into multiple nodes:

1. **Extension Node** - Shared prefix "dog"
```rust
Node::Extension(
    compact_encode([6,4,6,15,6,7], false),  // Shared "dog"
    Hash₂                                    // Points to branch
)
```

2. **Branch Node** - Split point
```rust
Node::Branch(
    children: [
        None, None, None, None, None, None,
        Some(Hash₃),  // Index 6 → "e" of "doge"
        None, None, None, None, None, None, None, None, None
    ],
    value: Some("puppy")  // "dog" value here
)
```

3. **Leaf Node** - Remaining "e"
```rust
Node::Leaf(
    compact_encode([6,5], true),  // "e" = [6,5]
    vec![99, 111, 105, 110]       // "coin"
)
```

**Storage:**
```
┌───────────────────────────────────────────────────┐
│ HashMap<Hash, Node>                               │
├───────────────────────────────────────────────────┤
│ Hash₁ → Extension([6,4,6,15,6,7], Hash₂)         │ ← Root
│ Hash₂ → Branch([..., [6]=Hash₃, ...], "puppy")   │
│ Hash₃ → Leaf([6,5], "coin")                      │
└───────────────────────────────────────────────────┘
     ↑
Root = Hash₁
```

**Tree Visualization:**
```
                    Root (Hash₁)
                        │
                   Extension
                   path: [6,4,6,15,6,7]
                   (spells "dog" in nibbles)
                   child: Hash₂
                        │
                        ↓
                    Branch (Hash₂)
                   ┌────┴────┐
           Index 6 │         │ 
           (for "e")│        │ value="puppy" stored here
                   ↓         ↓
              Leaf (Hash₃)   "dog" ends here
              path: [6,5]    
              value: "coin"
              ("doge")
```

---

## Hash Computation Details

### How Hash₃ is Computed (Leaf Node)

```rust
// 1. Create the leaf
let leaf = Node::Leaf(
    compact_encode([6,5], true),
    b"coin".to_vec()
);

// 2. RLP encode
let rlp = RlpEncodable::rlp_encode(leaf);
// Result: [0xc?, compact([6,5]), 0x84, 'c','o','i','n']

// 3. Hash it
Hash₃ = keccak256(rlp);
// Result: 32 bytes like [0x7a, 0xb2, ..., 0x3f]
```

### How Hash₂ is Computed (Branch Node)

```rust
// 1. Create the branch
let branch = Node::Branch(
    [None, None, ..., Some(Hash₃), ...],
    Some(b"puppy".to_vec())
);

// 2. RLP encode
let rlp = RlpEncodable::rlp_encode(branch);
// Result: [0xf?, [], [], ..., Hash₃, ..., 'puppy']
//         17 items: 16 children + 1 value

// 3. Hash it
Hash₂ = keccak256(rlp);
```

### How Hash₁ is Computed (Extension/Root)

```rust
// 1. Create the extension
let ext = Node::Extension(
    compact_encode([6,4,6,15,6,7], false),
    Hash₂
);

// 2. RLP encode
let rlp = RlpEncodable::rlp_encode(ext);
// Result: [0xc?, compact_path, Hash₂_bytes]

// 3. Hash it
Hash₁ = keccak256(rlp);
// This becomes the new root!
```

---

## Verification Example

### Question: Does "doge" → "coin" exist?

**With Full Trie (What your code does):**

```rust
1. Start with root Hash₁
2. Fetch storage[Hash₁] → Extension node
   - Path: [6,4,6,15,6,7]
   - Our key: [6,4,6,15,6,7,6,5]
   - Match first 6 nibbles ✓
   - Remaining: [6,5]
   - Follow child: Hash₂

3. Fetch storage[Hash₂] → Branch node
   - Next nibble: 6
   - Check children[6] → Hash₃
   - Remaining: [5]
   - Follow child: Hash₃

4. Fetch storage[Hash₃] → Leaf node
   - Path: [6,5]
   - Our remaining: [5]
   - Match ✓
   - Return value: "coin" ✓
```

**Complexity:** 3 lookups (O(log₁₆(n)))

---

### With Merkle Proof (Light Client)

**Prover sends:**
```
Proof = [
    Extension([6,4,6,15,6,7], Hash₂),
    Branch([..., Hash₃, ...], Some("puppy")),
    Leaf([6,5], "coin")
]

Trusted Root: Hash₁
```

**Verifier checks:**
```rust
1. Hash the leaf:
   RLP(Leaf([6,5], "coin")) → hash it → computed_hash₃
   
2. Check branch contains computed_hash₃:
   Branch.children[6] == computed_hash₃ ✓
   
3. Hash the branch:
   RLP(Branch(...)) → hash it → computed_hash₂
   
4. Check extension contains computed_hash₂:
   Extension.child == computed_hash₂ ✓
   
5. Hash the extension:
   RLP(Extension(...)) → hash it → computed_hash₁
   
6. Compare with trusted root:
   computed_hash₁ == Hash₁ ✓
```

**If all checks pass → "doge"="coin" is proven!**

**Complexity:** 3 hashes (constant time, independent of trie size!)

---

## Storage Size Comparison

### Full Trie Storage

```
For 1,000,000 keys:
- ~1,000,000 leaf nodes
- ~500,000 internal nodes (branches + extensions)
- ~1.5M × 32 bytes = ~48 MB in hashes alone
- Plus actual node data
```

### Merkle Proof

```
For any single key:
- ~log₁₆(1,000,000) ≈ 5 nodes
- ~5 × 100 bytes ≈ 500 bytes
- 96,000× smaller!
```

---

## Key Insights

1. **Content-Addressed**: Nodes stored by hash, not location
2. **Immutable Structure**: Changing any value changes all parent hashes up to root
3. **Efficient Verification**: Small proofs verify specific keys
4. **Deduplication**: Identical subtrees share storage
5. **Historical Snapshots**: Old roots remain valid if nodes kept

---

## Next Steps

To implement Merkle proofs in your code, we'd add:

```rust
// In trie.rs
pub fn generate_proof(&self, key: &[u8]) -> Vec<Node> {
    // Collect nodes along the path
}

pub fn verify_proof(
    key: &[u8],
    value: &[u8],
    proof: &[Node],
    root: Hash
) -> bool {
    // Verify hashes match
}
```

This would enable light client verification! 🚀

