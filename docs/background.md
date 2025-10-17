# Trie, Merkle Tree and Merkle Patricia Tree

## 1. Trie

A trie (pronounced "try") is a tree-based data structure for storing key-value pairs where keys share common prefixes.

**Key Properties**

* Each node represents a character/digit in the key
* Path from root to leaf spell out complete key
* Common prefixes are shared  (space-efficient for similar keys)

**Example**:

```
Storing: "dog", "doge", "dodge"

                            root
                             |
                             d
                             |
                             o
                             |
                             g ← "dog" value here
                            / \
    "doge" value here  ->  e   d
                               |
                               g           
                               |
                               e ← "dodge" value here
```

## 2. 🔐 Merkle Tree
A Merkle tree is a binary tree where:

* Leaf nodes contain data hashes
* Internal nodes contain hashes of their children
* The root hash represents the entire dataset

**Structure:**

```
          Root Hash
          /        \
    Hash(A,B)    Hash(C,D)
      /   \        /   \
   Hash(A) Hash(B) Hash(C) Hash(D)
     |       |       |       |
   Data A  Data B  Data C  Data D
```

**Key Properties:**

* Integrity: Any change to data changes the root hash
* Efficient Verification: Can prove data exists with log(n) hashes
* Merkle Proofs: Verify a piece of data without downloading everything

```
// To verify "Data B" is in the tree, you only need:
// 1. Hash(B)
// 2. Hash(A) (sibling)
// 3. Hash(C,D) (uncle)
// Then compute upward and compare to known root hash
```

## 3. 🎯 Merkle Patricia Tree (MPT)
The Merkle Patricia Tree combines the best of both worlds:

* Patricia Trie: Path compression (skips single-child chains)
* Merkle Tree: Cryptographic verification

**Three Node Types:**

1. Leaf Node [encoded_path, value]

```
Stores actual data at the end of a path
Example: Key "dog" → Value "puppy"
```

2. Extension Node [encoded_path, child_hash]

```
Compresses shared prefixes
Instead of: A → B → C → D → [value]
Use:        ABC → D → [value]
```

3. Branch Node [child₀, child₁, ..., child₁₅, value]

```
16-way branching for hex digits (0-F)
Optional value slot for keys ending here
```

**Visual Example**:

Keys: "do" → "verb", "dog" → "puppy", "doge" → "coin"
```
After converting to hex nibbles:
"do"   = [6,4,6,f]
"dog"  = [6,4,6,f,6,7]
"doge" = [6,4,6,f,6,7,6,5]

Tree structure:
    Root (Extension)
    path: [6,4,6,f]
         |
    Branch (at position 6)
    ├─[value: "verb"]
    └─[6]: Extension
           path: [7]
               |
          Branch (at position 6)
          ├─[value: "puppy"]
          └─[5]: Leaf
                 value: "coin"
```

```
User Data                   NOT hashed/RLP'd
┌─────────────┐
│ key: "dog"  │──→ Convert to nibbles ──→ [6,4,6,15,6,7]
│ val: "puppy"│──→ Keep as-is ──────────→ [112,117,112,112,121]
└─────────────┘

        ↓

Create Node (contains raw data)
┌──────────────────────────────────┐
│ Node::Leaf(                      │
│   path: compact([6,4,6,15,6,7]), │
│   value: [112,117,112,112,121]   │ ← Raw "puppy"!
│ )                                │
└──────────────────────────────────┘

        ↓ node.hash()

RLP Encode the Node              ← THIS is where RLP happens!
┌──────────────────────────────────┐
│ [0xc8, 0x20, 0x64, ..., 'p', ...]│
└──────────────────────────────────┘

        ↓ keccak256()

Hash of Node                     ← THIS is where hashing happens!
┌──────────────────────────────────┐
│ [0xab, 0xcd, ..., 0xef]  32 bytes│
└──────────────────────────────────┘

        ↓

Storage: HashMap<Hash, Node>
┌─────────────────────────────────────┐
│ 0xabcd...ef → Node::Leaf(...)       │ ← Node still has raw data!
└─────────────────────────────────────┘
```