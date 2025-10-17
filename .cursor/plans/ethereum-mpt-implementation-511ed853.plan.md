<!-- 511ed853-3fdd-4ec4-ae8a-2cc6d2373432 f3f3b56a-5519-4717-804b-cc809efbd193 -->
# Add Merkle Proof Support to MPT

## Overview

Extend the existing Merkle Patricia Trie implementation with proof generation and verification capabilities. This enables lightweight verification where a client can prove a key exists using only ~5 nodes instead of the entire trie.

## What is a Merkle Proof?

A proof is a sequence of nodes along the path from root to a leaf that allows anyone with just the root hash to verify that a specific key-value pair exists in the trie.

## Implementation Steps

### 1. Create Proof Module (`src/proof.rs`)

Add a new module for proof-related structures and functions:

**Structures:**

- `MerkleProof` struct containing:
  - `key: Vec<u8>` - The key being proven
  - `value: Vec<u8>` - The value for that key
  - `nodes: Vec<Node>` - Path of nodes from root to leaf
  - `root_hash: Hash` - The root hash to verify against

**Functions:**

- `new()` - Constructor
- `verify(&self) -> bool` - Verifies the proof is valid

### 2. Add Proof Generation to Trie (`src/trie.rs`)

Add method to collect nodes along the path:

```rust
pub fn generate_proof(&self, key: &[u8]) -> Option<MerkleProof>
```

**Algorithm:**

1. Convert key to nibbles
2. Traverse from root to leaf (same as get)
3. Collect each node visited along the path
4. Return proof with collected nodes + value

### 3. Implement Standalone Verification Function

Add public function that doesn't require the full trie:

```rust
pub fn verify_proof(proof: &MerkleProof) -> bool
```

**Algorithm:**

1. Start with the deepest node (leaf)
2. Hash it and verify it contains the expected value
3. Work backwards through parent nodes
4. For each parent, verify it contains the child's hash
5. Hash the root node and compare to proof.root_hash

### 4. Update `lib.rs`

Export the new proof functionality:

```rust
pub mod proof;
pub use proof::{MerkleProof, verify_proof};
```

### 5. Add Comprehensive Tests

**Test cases:**

- Basic proof generation and verification
- Proof for non-existent key (should return None)
- Proof with multiple keys (different paths)
- Invalid proof detection (tampered value, wrong root)
- Edge cases (empty trie, single key, deep paths)

### 6. Create Example Demonstration

Add `examples/proof_demo.rs` showing:

- Full node generating proof
- Light client verifying proof
- Size comparison (full trie vs proof)
- Invalid proof handling

## Technical Details

### Proof Structure

For key "doge" → "coin":

```
Proof contains:
- Extension node ([6,4,6,15,6,7], Hash₂)
- Branch node with children and value
- Leaf node ([6,5], "coin")

Verification:
1. Hash(Leaf) → Hash₃
2. Verify Hash₃ in Branch.children[6]
3. Hash(Branch) → Hash₂
4. Verify Hash₂ in Extension.child
5. Hash(Extension) → Hash₁
6. Verify Hash₁ == root_hash ✓
```

### Key Insights

- Proof size: O(log₁₆(n)) nodes
- Verification: O(k) where k = key length
- No need for full trie storage
- Critical for blockchain light clients

## Files to Modify

```
src/
├── lib.rs           # Add proof module export
├── trie.rs          # Add generate_proof method
└── proof.rs         # NEW: Proof struct and verification

examples/
└── proof_demo.rs    # NEW: Demonstration

tests/ or src/main.rs
└── Add proof tests
```

## Success Criteria

- ✅ Can generate proof for any key in trie
- ✅ Can verify valid proofs without full trie
- ✅ Detects invalid/tampered proofs
- ✅ Proof size << full trie size
- ✅ Comprehensive test coverage

### To-dos

- [ ] Add required crates (tiny-keccak, rlp, hex) to Cargo.toml
- [ ] Implement nibbles.rs with key encoding and compact encoding utilities
- [ ] Create node.rs with Node enum and RLP encoding/hashing logic
- [ ] Implement trie.rs with core operations (insert, get, delete, root_hash)
- [ ] Create lib.rs to export public API