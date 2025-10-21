# Pretty Printing MPT Trie

This document describes the visualization functions added to the Merkle Patricia Trie implementation.

## Overview

Two main functions have been added to help visualize and debug the MPT structure:

1. **`print_tree()`** - Pretty prints the trie as a hierarchical tree structure
2. **`print_storage()`** - Displays all nodes stored in the internal HashMap

## Usage

```rust
use mpt::MerklePatriciaTrie;

fn main() {
    let mut trie = MerklePatriciaTrie::new();
    
    // Insert some data
    trie.insert(b"do", b"verb".to_vec());
    trie.insert(b"dog", b"puppy".to_vec());
    trie.insert(b"doge", b"coin".to_vec());
    
    // Print as tree structure
    trie.print_tree();
    
    // Print storage details
    trie.print_storage();
}
```

## Function Details

### `print_tree()`

Displays the trie as a hierarchical tree structure using Unicode box-drawing characters.

**Features:**
- Shows the root hash at the top
- Recursively displays all nodes in tree form
- For each node, shows:
  - Node type (Empty, Leaf, Extension, Branch)
  - Path information (in nibbles/hex)
  - Values (as strings if printable, otherwise hex)
  - Node hash (truncated for readability)
- Handles branch nodes by showing all non-empty children

**Example Output:**
```
╔═══════════════════════════════════════════════════════════════
║ Merkle Patricia Trie Structure
╠═══════════════════════════════════════════════════════════════
║ Root Hash: 0x2a928f40...b2019c7d
╚═══════════════════════════════════════════════════════════════

└── Extension
       Path: 6
       Hash: 0x2a928f40...b2019c7d
    └── Branch
           Hash: 0x8da4fbae...85abac01
           ├[4]
           │   └── Extension
           │          Path: 6f
           │          Hash: 0x5e0b4dc6...982d7ea1
           │       └── Branch
           │              Value: "verb"
           │              Hash: 0x47ab9517...21fcb30a
           ...
```

### `print_storage()`

Displays all nodes in the internal storage HashMap with complete details.

**Features:**
- Shows total node count and root hash
- Lists each node with:
  - Full 32-byte hash (not truncated)
  - Node type
  - All relevant data (paths, values, children)
- For branch nodes, lists all non-empty children with their indices
- Useful for debugging and understanding the internal structure

**Example Output:**
```
╔═══════════════════════════════════════════════════════════════
║ Trie Storage Contents
╠═══════════════════════════════════════════════════════════════
║ Total nodes: 13
║ Root hash: 0x2a928f40...b2019c7d
╚═══════════════════════════════════════════════════════════════

Node #1
  Hash: 0x2a928f4070f07fa2dc19f64b956ad0a8bb4830c5977035b5b22ef6d4b2019c7d
  Type: Extension
  Path (nibbles): 6
  Path (encoded): 0x16
  Child: 0x8da4fbae...85abac01

Node #2
  Hash: 0x014f07ed95e2e028804d915e0dbd4ed451e394e1acfd29e463c11a060b2ddef7
  Type: Leaf
  Path (nibbles): 646f
  Path (encoded): 0x20646f
  Value: "verb"
...
```

## Node Type Explanations

### Empty Node
- Represents an empty/null node
- Used as placeholder for non-existent data
- Has a specific hash (Keccak256 of empty bytes)

### Leaf Node
- Stores actual key-value data
- Contains:
  - Encoded path (compact encoding with terminator flag)
  - Value (raw bytes)
- Terminates a path in the trie

### Extension Node
- Compresses long shared path prefixes
- Contains:
  - Encoded path (compact encoding)
  - Reference (hash) to child node
- Optimizes storage for common prefixes

### Branch Node
- Has 16 children (one for each hex digit 0-f)
- Can optionally store a value at the branch itself
- Represents a divergence point in the trie

## Helper Functions

The following internal helper functions are used by the printing functions:

- `nibbles_to_hex()` - Converts nibbles to hex string
- `format_value()` - Formats values as strings or hex
- `hex_bytes()` - Converts bytes to hex string
- `hex_truncated()` - Creates truncated hash display (first 8 + last 8 chars)
- `hex_full()` - Full hash in hex format

## Examples

Run the included examples:

```bash
# Simple demonstration of printing functions
cargo run --example print_demo

# Full storage demonstration including prints
cargo run --example storage_demo
```

## Use Cases

### 1. Debugging
- Visualize the trie structure during development
- Verify that insertions/deletions work correctly
- Check node types and path compression

### 2. Learning
- Understand how MPT works internally
- See how different keys create different structures
- Learn about path compression and branching

### 3. Testing
- Verify expected trie shapes
- Check storage efficiency
- Debug edge cases

## Tips

1. **For large tries**, use `print_tree()` to see the hierarchical structure without overwhelming detail
2. **For debugging specific nodes**, use `print_storage()` to see full hashes and details
3. **Values are shown as strings** when possible (UTF-8 printable), otherwise as hex
4. **Paths are shown as hex nibbles** - each character is a hex digit (0-f)
5. **Hashes are truncated** in tree view but full in storage view

## Performance Note

These functions are intended for debugging and visualization only. They traverse the entire trie and format output, which can be slow for large tries. Do not use in production code paths.

