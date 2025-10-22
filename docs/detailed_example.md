# Detailed MPT Example: Step-by-Step Evolution

## Complete Walkthrough: Inserting Keys into MPT

Let's insert keys one-by-one and watch the tree structure evolve, showing:
- When extension nodes are created
- How common prefixes are determined
- Actual hash computation
- Why certain structures are chosen

---

## Starting Point: Empty Trie

```rust
let mut trie = MerklePatriciaTrie::new();
```

**State:**
```
Root: keccak256([]) = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
Storage: {} (empty)
```

**Root hash for empty trie** is the hash of empty bytes (this is a standard in Ethereum).

---

## Insert #1: "dog" → "puppy"

```rust
trie.insert(b"dog", b"puppy".to_vec());
```

### Step 1: Convert Key to Nibbles

```
"dog" = [0x64, 0x6f, 0x67] in bytes

Convert each byte to 2 nibbles:
0x64 → [6, 4]
0x6f → [6, 15]  (0xf = 15)
0x67 → [6, 7]

Result: [6, 4, 6, 15, 6, 7]
```

### Step 2: Create Leaf Node

Since the trie is empty, we create a single leaf node containing the entire path:

```rust
Node::Leaf(
    compact_encode([6, 4, 6, 15, 6, 7], true),  // is_leaf = true
    b"puppy".to_vec()
)
```

**Compact Encoding of [6, 4, 6, 15, 6, 7]:**
- Length: 6 (even)
- Is leaf: true
- Flag: 0010 (2) = leaf + even length
- Result: [0x20, 0x64, 0x6f, 0x67]
  - 0x20 = flags (0010 0000)
  - Then nibbles paired back to bytes

### Step 3: RLP Encode the Node

```
List with 2 elements:
[
    [0x20, 0x64, 0x6f, 0x67],  // Compact-encoded path
    [0x70, 0x75, 0x70, 0x70, 0x79]  // "puppy"
]

RLP encoding:
0xc8  ← List marker (0xc0 + 8 bytes)
  0x84, 0x20, 0x64, 0x6f, 0x67  ← Path (4 bytes + 1 length byte)
  0x85, 0x70, 0x75, 0x70, 0x70, 0x79  ← Value (5 bytes + 1 length byte)
```

### Step 4: Compute Hash

```rust
let rlp = [0xc8, 0x84, 0x20, 0x64, 0x6f, 0x67, 0x85, 0x70, 0x75, 0x70, 0x70, 0x79];
let hash = keccak256(rlp);
// Result: 0xed6e08740e4a267eca9d4740f71f573e9aabbcc739b16a2fa6c1baed5ec21278
```

### State After Insert #1

```
Tree Structure:
    Root
     ↓
   Leaf
   path: [6,4,6,15,6,7]
   value: "puppy"

Storage:
├─ 0xed6e0874... → Leaf([6,4,6,15,6,7], "puppy")

Root: 0xed6e0874...
```

**Key Point:** With only one key, we use a **single Leaf node** - no extension needed!

---

## Insert #2: "doge" → "coin"

```rust
trie.insert(b"doge", b"coin".to_vec());
```

### Step 1: Convert Key to Nibbles

```
"doge" = [0x64, 0x6f, 0x67, 0x65]

0x64 → [6, 4]
0x6f → [6, 15]
0x67 → [6, 7]
0x65 → [6, 5]

Result: [6, 4, 6, 15, 6, 7, 6, 5]
```

### Step 2: Compare with Existing Path

```
Existing path: [6, 4, 6, 15, 6, 7]       (from "dog")
New path:      [6, 4, 6, 15, 6, 7, 6, 5] (from "doge")

Common prefix: [6, 4, 6, 15, 6, 7]       (length = 6)
Divergence point: position 6
```

**How common prefix is found:**
```rust
fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b.iter())
        .take_while(|(x, y)| x == y)
        .count()
}

common_prefix_len([6,4,6,15,6,7], [6,4,6,15,6,7,6,5])
// Compares: (6,6)✓ (4,4)✓ (6,6)✓ (15,15)✓ (6,6)✓ (7,7)✓ (end of first array)
// Returns: 6
```

### Step 3: Restructure the Tree

Since we have a common prefix, we need to split the tree:

**After prefix [6,4,6,15,6,7]:**
- Old path ("dog"): [] (nothing left, value at branch)
- New path ("doge"): [6, 5] (two more nibbles)

**New Structure:**

```
Extension Node:
├─ path: [6,4,6,15,6,7] (shared prefix)
└─ child: → Branch Node

Branch Node:
├─ value: Some("puppy")  ← "dog" ends here!
└─ children[6]: → Leaf Node for "doge"

Leaf Node:
├─ path: [5] (remaining nibble)
└─ value: "coin"
```

### Step 4: Create Nodes and Compute Hashes

**Leaf Node for "doge":**
```rust
Node::Leaf(compact_encode([5], true), b"coin".to_vec())
// [5] is odd length
// compact: 0x35 ([0011 0101] = leaf + odd + 5)

RLP: [0xc6, 0x35, 0x84, 0x63, 0x6f, 0x69, 0x6e]
Hash_Leaf = keccak256(RLP) = 0xabc123... (example)
```

**Branch Node:**
```rust
Node::Branch(
    [None, None, ..., Some(Hash_Leaf), ..., None],  // children[6] = Hash_Leaf
    Some(b"puppy".to_vec())  // value
)

RLP: 17-element list
[
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80,  // 6 empty children
    0xa0, 0xab, 0xc1, 0x23, ...,         // child[6]: Hash_Leaf (32 bytes)
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,  // 9 more empty
    0x85, 0x70, 0x75, 0x70, 0x70, 0x79   // value: "puppy"
]

Hash_Branch = keccak256(RLP) = 0xdef456... (example)
```

**Extension Node:**
```rust
Node::Extension(
    compact_encode([6,4,6,15,6,7], false),  // is_leaf = false
    Hash_Branch
)

// [6,4,6,15,6,7] is even length (6)
// compact: [0x00, 0x64, 0x6f, 0x67]
//   0x00 = [0000 0000] = extension + even + padding

RLP: [0xe4, 0x83, 0x00, 0x64, 0x6f, 0x67, 0xa0, 0xde, 0xf4, 0x56, ...]
Hash_Ext = keccak256(RLP) = 0x123abc...

Root = Hash_Ext
```

### State After Insert #2

```
Tree Structure:
         Root (Hash_Ext)
           ↓
      Extension
      path: [6,4,6,15,6,7]
      child: Hash_Branch
           ↓
         Branch
      ├─ value: "puppy" (dog)
      └─ children[6]: Hash_Leaf
                        ↓
                      Leaf
                      path: [5]
                      value: "coin" (doge)

Storage:
├─ Hash_Ext   → Extension([6,4,6,15,6,7], Hash_Branch)
├─ Hash_Branch → Branch([..., children[6]=Hash_Leaf, ...], "puppy")
└─ Hash_Leaf  → Leaf([5], "coin")

Root: Hash_Ext
```

**Key Point:** Extension node created because **common prefix exists** (length > 0)

---

## Insert #3: "do" → "verb"

```rust
trie.insert(b"do", b"verb".to_vec());
```

### Step 1: Convert Key

```
"do" = [0x64, 0x6f]
Nibbles: [6, 4, 6, 15]
```

### Step 2: Traverse Existing Tree

```
Start at Root (Extension)
Extension path: [6,4,6,15,6,7]
Our path:       [6,4,6,15]

Common: [6,4,6,15] (length = 4)
Extension has more: [6,7] remaining
Our path: [] (nothing remaining)
```

**Decision:** Need to **split the extension node**!

### Step 3: Split Extension Node

Original extension: [6,4,6,15,6,7] → Branch

New structure needed:
```
Extension([6,4,6,15]) → New Branch
                         ├─ value: "verb" (do ends here!)
                         └─ children[6] → Extension([7]) → Old Branch (for dog/doge)
```

### Step 4: Create New Nodes

**Extension([7]) → Old Branch:**
```rust
Node::Extension(compact_encode([7], false), Hash_OldBranch)
// [7] is odd: 0x17 ([0001 0111] = extension + odd + 7)

Hash_Ext7 = keccak256(RLP)
```

**New Branch:**
```rust
Node::Branch(
    [None, None, ..., Some(Hash_Ext7), ...],  // children[6]
    Some(b"verb".to_vec())  // "do" ends here
)

Hash_NewBranch = keccak256(RLP)
```

**New Root Extension:**
```rust
Node::Extension(compact_encode([6,4,6,15], false), Hash_NewBranch)

Hash_NewRoot = keccak256(RLP)
```

### State After Insert #3

```
Tree Structure:
            Root
             ↓
        Extension
        path: [6,4,6,15]
        child: ↓
           Branch
        ├─ value: "verb" (do)
        └─ children[6]: ↓
               Extension
               path: [7]
               child: ↓
                  Branch
               ├─ value: "puppy" (dog)
               └─ children[6]: ↓
                      Leaf
                      path: [5]
                      value: "coin" (doge)

Storage:
├─ Hash1 → Extension([6,4,6,15], Hash2)
├─ Hash2 → Branch([..., children[6]=Hash3, ...], "verb")
├─ Hash3 → Extension([7], Hash4)
├─ Hash4 → Branch([..., children[6]=Hash5, ...], "puppy")
└─ Hash5 → Leaf([5], "coin")

Root: Hash1
```

**Key Point:** Extension split because new key is **prefix of existing path**

---

## Insert #4: "horse" → "stallion"

```rust
trie.insert(b"horse", b"stallion".to_vec());
```

### Step 1: Convert Key

```
"horse" = [0x68, 0x6f, 0x72, 0x73, 0x65]
Nibbles: [6, 8, 6, 15, 7, 2, 7, 3, 6, 5]
```

### Step 2: Compare with Root Extension

```
Extension path: [6,4,6,15]
Our path:       [6,8,6,15,7,2,7,3,6,5]

Common: [6] (length = 1)
```

**Only first nibble matches!**

### Step 3: Split at First Nibble

Need to create a branch after [6]:

```
Extension([6]) → New Branch
                  ├─ children[4] → Rest of old tree (do/dog/doge)
                  └─ children[8] → New path for "horse"
```

### Step 4: Restructure

**For old tree (children[4]):**
Everything after [6] from old paths starts with [4,...], so:
```rust
Extension([4,6,15]) → (rest of tree)
```

**For "horse" (children[8]):**
After [6], "horse" has [8,6,15,7,2,7,3,6,5]:
```rust
Leaf([8,6,15,7,2,7,3,6,5], "stallion")
```

### State After Insert #4

```
Tree Structure:
         Root
          ↓
     Extension
     path: [6]
     child: ↓
        Branch
     ├─ children[4]: → Extension([4,6,15]) → (do/dog/doge subtree)
     └─ children[8]: → Leaf([8,6,15,7,2,7,3,6,5], "stallion")

Full tree visualization:

                    Root Extension([6])
                            ↓
                         Branch
                    ┌──────┴──────┐
            children[4]      children[8]
                    ↓              ↓
        Extension([4,6,15])      Leaf
                ↓                path:[8,6,15,7,2,7,3,6,5]
            Branch               value:"stallion"
        ├─ value:"verb"
        └─ children[6]
                ↓
           Extension([7])
                ↓
              Branch
          ├─ value:"puppy"
          └─ children[6]
                  ↓
                Leaf
                path:[5]
                value:"coin"

Storage now has 8 nodes total
Root: Hash of Extension([6])
```

**Key Point:** Extension shortened to **only the common part** [6]

---

## Key Decision Rules

### When to Create Extension Node?

**Rule:** Create extension node when `common_prefix_length > 0`

```rust
if common_len > 0 {
    // Create extension for the common prefix
    Extension(common_prefix) → child
} else {
    // No common prefix, start with branch
    child
}
```

### When to Create Branch Node?

**Rule:** Create branch when paths **diverge** (different next nibbles)

```rust
if path1[common_len] != path2[common_len] {
    // Create branch
    Branch {
        children[path1[common_len]]: ...,
        children[path2[common_len]]: ...,
    }
}
```

### When to Create Leaf Node?

**Rule:** Create leaf for **remaining path** after last branch

```rust
if remaining_path.len() > 0 {
    // Store remaining path in leaf
    Leaf(remaining_path, value)
} else {
    // No remaining path, value goes in branch
    Branch { value: Some(value), ... }
}
```

---

## Hash Root Computation Summary

**Every time a node is created or modified:**

```rust
fn store_node(&mut self, node: Node) -> Hash {
    // 1. RLP encode the node
    let rlp = node.encode_raw();
    
    // 2. Hash the RLP encoding
    let hash = keccak256(&rlp);
    
    // 3. Store mapping
    self.storage.insert(hash, node);
    
    // 4. Return hash (used as reference)
    hash
}
```

**Root hash** = hash of the topmost node (could be Leaf, Extension, or Branch)

**Chain of hashes:**
```
Leaf → hash₁
Branch(contains hash₁) → hash₂
Extension(contains hash₂) → hash₃ = ROOT
```

Any change to any node → changes all parent hashes up to root!

---

## Summary: Default Structures

### Single Key
```
Default: Just a Leaf
Leaf(entire_path, value)
```

### Two Keys with Common Prefix
```
If common_len > 0:
  Extension(common) → Branch → Leaves
Else:
  Branch → Leaves
```

### Multiple Keys
```
Extension(longest_common_prefix) → Branch tree → More extensions/branches/leaves
```

### Optimization Rules
1. **Path compression**: Use Extension for any shared prefix
2. **Value placement**: Put value in Branch if path ends there, else Leaf
3. **Single child**: Branch with one child gets collapsed into Extension
4. **No children + no value**: Node deleted, parents normalized

---

## Summary Table

| Keys in Trie | Structure | Extension Used? | Why |
|--------------|-----------|----------------|-----|
| "dog" only | Leaf | No | Single key, no sharing needed |
| + "doge" | Ext→Branch→Leaf | Yes | Common prefix "dog" = 6 nibbles |
| + "do" | Ext→Branch→Ext→... | Yes (2×) | Multiple shared prefixes at different levels |
| + "horse" | Ext→Branch (splits) | Yes (1×) | Only "h" (1 nibble) shared with others |

**Default behavior:** Always maximize path compression by creating Extension nodes for any shared prefix length > 0.

This is exactly what your `common_prefix_len` function in `nibbles.rs` enables!

---

## Verification with Your Code

You can verify this behavior by running:

```rust
let mut trie = MerklePatriciaTrie::new();

println!("Empty root: {}", hex::encode(trie.root_hash()));

trie.insert(b"dog", b"puppy".to_vec());
println!("After 'dog': {}", hex::encode(trie.root_hash()));

trie.insert(b"doge", b"coin".to_vec());
println!("After 'doge': {}", hex::encode(trie.root_hash()));

trie.insert(b"do", b"verb".to_vec());
println!("After 'do': {}", hex::encode(trie.root_hash()));

trie.insert(b"horse", b"stallion".to_vec());
println!("After 'horse': {}", hex::encode(trie.root_hash()));
```

Each insert will create a different root hash as the tree structure changes!

