//! Ethereum-style Merkle Patricia Trie Implementation
//! 
//! This library provides a complete implementation of the Modified Merkle Patricia Trie
//! used in Ethereum, with hex-based keys and three node types: Branch, Extension, and Leaf.
//! 
//! # Example
//! 
//! ```
//! use mpt::MerklePatriciaTrie;
//! 
//! let mut trie = MerklePatriciaTrie::new();
//! 
//! // Insert key-value pairs
//! trie.insert(b"dog", b"puppy".to_vec());
//! trie.insert(b"doge", b"coin".to_vec());
//! 
//! // Retrieve values
//! assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
//! 
//! // Get root hash
//! let root = trie.root_hash();
//! println!("Root hash: {:?}", hex::encode(root));
//! 
//! // Delete keys
//! trie.delete(b"dog");
//! assert_eq!(trie.get(b"dog"), None);
//! ```

pub mod nibbles;
pub mod node;
pub mod trie;

pub use trie::MerklePatriciaTrie;
pub use node::{Node, Hash};

