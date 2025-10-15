# Ethereum-style Merkle Patricia Trie Implementation

## Overview

Implement a Modified Merkle Patricia Trie following Ethereum's specification with hex-based keys, three node types, and basic CRUD operations.

## Implementation Steps

### 1. Add Dependencies

Update `Cargo.toml` to include:

- `tiny-keccak` for Keccak-256 hashing
- `rlp` for RLP encoding/decoding
- `hex` for hex utilities

### 2. Core Data Structures (`src/node.rs`)

Define the node types:

- `Node` enum with variants: Empty, Leaf, Extension, Branch
- `Leaf`: stores (key_end, value)
- `Extension`: stores (shared_nibbles, child_hash)
- `Branch`: stores 16 children + optional value
- Node hashing logic

### 3. Nibble Path Utilities (`src/nibbles.rs`)

Implement key encoding:

- Convert bytes to nibbles (hex digits)
- Compact encoding with terminator flags
- Path matching and prefix operations

### 4. Merkle Patricia Trie (`src/trie.rs`)

Main trie structure with:

- In-memory storage using HashMap<hash, Node>
- `insert(key, value)` - add/update key-value pairs
- `get(key)` - retrieve values by key
- `delete(key)` - remove keys
- `root_hash()` - compute the current root hash
- Helper methods for node traversal and updates

### 5. Testing (`src/main.rs` and tests)

Create comprehensive tests:

- Basic insert/get/delete operations
- Root hash verification
- Edge cases (empty trie, single item, deletions)
- Multiple insertions to test branch/extension node creation

## Key Technical Details

- **Nibble representation**: Each byte becomes 2 hex digits (nibbles)
- **Node storage**: Nodes are stored by their hash (Keccak-256 of RLP-encoded node)
- **Path compression**: Extension nodes compress shared prefixes
- **Branch nodes**: 16-element array for hex digits 0-F plus optional value at position 16

## Module Structure

```
src/
├── main.rs          # Entry point and tests
├── lib.rs           # Module declarations
├── node.rs          # Node types and encoding
├── nibbles.rs       # Path/key utilities
└── trie.rs          # Main trie implementation
```

## To-dos

- [x] Add required crates (tiny-keccak, rlp, hex) to Cargo.toml
- [x] Implement nibbles.rs with key encoding and compact encoding utilities
- [x] Create node.rs with Node enum and RLP encoding/hashing logic
- [x] Implement trie.rs with core operations (insert, get, delete, root_hash)
- [x] Create lib.rs to export public API
- [x] Write comprehensive tests in main.rs demonstrating all operations

## Implementation Complete ✅

All components have been successfully implemented and tested. The project includes:
- 5 source modules with full documentation
- 30+ passing tests
- Working demo application
- Comprehensive README

