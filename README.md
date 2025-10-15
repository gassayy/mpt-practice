# Merkle Patricia Trie Implementation

A toy implementation of the Ethereum-style Modified Merkle Patricia Trie in Rust, featuring hex-based keys and the three node types: Branch, Extension, and Leaf.

## Overview

This project implements the data structure used by Ethereum to efficiently store and verify key-value pairs. The Merkle Patricia Trie combines the benefits of:
- **Merkle Trees**: Cryptographic verification through hashing
- **Patricia Tries**: Efficient storage through path compression
- **Modified Structure**: Optimized for blockchain use with extension nodes

## Features

- ✅ **Basic Operations**: Insert, Get, Delete
- ✅ **Root Hash Computation**: Keccak-256 based cryptographic hashing
- ✅ **RLP Encoding**: Recursive Length Prefix encoding for nodes
- ✅ **Three Node Types**:
  - **Leaf Nodes**: Store actual key-value pairs
  - **Extension Nodes**: Compress shared path prefixes
  - **Branch Nodes**: 16-way branching for hex digits
- ✅ **Comprehensive Test Suite**: 30+ tests covering all functionality

## Installation

This project requires Rust. If you don't have Rust installed, get it from [rustup.rs](https://rustup.rs/).

```bash
# Clone or navigate to the project directory
cd mpt

# Build the project
cargo build

# Run tests
cargo test

# Run the demo
cargo run
```

## Usage

### Basic Example

```rust
use mpt::MerklePatriciaTrie;

fn main() {
    let mut trie = MerklePatriciaTrie::new();
    
    // Insert key-value pairs
    trie.insert(b"dog", b"puppy".to_vec());
    trie.insert(b"doge", b"coin".to_vec());
    
    // Retrieve values
    assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
    
    // Get root hash
    let root = trie.root_hash();
    println!("Root hash: {}", hex::encode(root));
    
    // Delete keys
    trie.delete(b"dog");
    assert_eq!(trie.get(b"dog"), None);
}
```

### API Reference

#### `MerklePatriciaTrie::new()`
Creates a new empty trie.

#### `insert(key: &[u8], value: Vec<u8>)`
Inserts or updates a key-value pair in the trie.

#### `get(key: &[u8]) -> Option<Vec<u8>>`
Retrieves a value by key. Returns `None` if the key doesn't exist.

#### `delete(key: &[u8])`
Removes a key from the trie.

#### `root_hash() -> Hash`
Returns the current root hash of the trie (32 bytes).

## Project Structure

```
src/
├── main.rs          # Demo application and integration tests
├── lib.rs           # Public API exports
├── nibbles.rs       # Nibble/hex key encoding utilities
├── node.rs          # Node types and RLP encoding
└── trie.rs          # Main trie implementation
```

## Technical Details

### Node Types

1. **Empty Node**: Represents absence of data (hash of empty bytes)

2. **Leaf Node**: `[encoded_path, value]`
   - Stores the remaining key path and the actual value
   - Path is compact-encoded with terminator flag

3. **Extension Node**: `[encoded_path, child_hash]`
   - Stores shared prefix and reference to child node
   - Used for path compression

4. **Branch Node**: `[child_0, ..., child_15, value]`
   - 16 slots for hex digit children (0-F)
   - Optional value slot for keys ending at this node

### Key Encoding

Keys are converted to nibbles (4-bit hex digits):
- Each byte `0xAB` becomes two nibbles `[10, 11]`
- Nibbles are then compact-encoded with flags:
  - Bit 0: Odd/even length flag
  - Bit 1: Leaf/extension flag

### Hashing

- Uses Keccak-256 (not standard SHA3)
- Nodes are RLP-encoded before hashing
- Hash references are 32 bytes

## Running Tests

The project includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_insert_and_get
```

## Example Output

```
=== Merkle Patricia Trie Demo ===

1. Created empty trie
   Root hash: c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470

2. Inserting key-value pairs:
   Inserted: 'do' -> 'verb'
   Root hash: 014f07ed95e2e028804d915e0dbd4ed451e394e1acfd29e463c11a060b2ddef7
   Inserted: 'dog' -> 'puppy'
   Root hash: ceb8464199416fb79eb65a870b984fde8b1667338ca8b02ed4fcb722641e4226
```

## References

- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)
- [Ethereum Wiki: Patricia Tree](https://eth.wiki/fundamentals/patricia-tree)
- [Merkle Patricia Trie Specification](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/)

## License

This is a course project implementation for educational purposes.

## Author

Course Project Implementation - 2025

