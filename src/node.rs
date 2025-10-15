use crate::nibbles::{compact_encode, compact_decode};
use rlp::{Encodable, Decodable, RlpStream, Rlp, DecoderError};
use tiny_keccak::{Hasher, Keccak};

/// Hash type used in the trie (32 bytes)
pub type Hash = [u8; 32];

/// Represents the different types of nodes in a Merkle Patricia Trie
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Empty node
    Empty,
    
    /// Leaf node: stores the remaining key path and value
    /// (encoded_path, value)
    Leaf(Vec<u8>, Vec<u8>),
    
    /// Extension node: stores a shared path prefix and a reference to the next node
    /// (encoded_path, child_hash)
    Extension(Vec<u8>, Hash),
    
    /// Branch node: 16 children (one for each hex digit) + optional value
    /// ([child_0, ..., child_15], optional_value)
    Branch(Box<[Option<Hash>; 16]>, Option<Vec<u8>>),
}

impl Node {
    /// Creates a new leaf node from nibbles and value
    pub fn new_leaf(nibbles: &[u8], value: Vec<u8>) -> Self {
        let encoded = compact_encode(nibbles, true);
        Node::Leaf(encoded, value)
    }
    
    /// Creates a new extension node from nibbles and child hash
    pub fn new_extension(nibbles: &[u8], child: Hash) -> Self {
        let encoded = compact_encode(nibbles, false);
        Node::Extension(encoded, child)
    }
    
    /// Creates a new empty branch node
    pub fn new_branch() -> Self {
        Node::Branch(Box::new([None; 16]), None)
    }
    
    /// Computes the hash of this node using Keccak-256
    pub fn hash(&self) -> Hash {
        let encoded = self.encode_raw();
        keccak256(&encoded)
    }
    
    /// Encodes the node to RLP format
    pub fn encode_raw(&self) -> Vec<u8> {
        let mut stream = RlpStream::new();
        self.rlp_append(&mut stream);
        stream.out().to_vec()
    }
    
    /// Decodes a node from RLP format
    pub fn decode_raw(data: &[u8]) -> Result<Self, DecoderError> {
        let rlp = Rlp::new(data);
        Self::decode(&rlp)
    }
}

impl Encodable for Node {
    fn rlp_append(&self, stream: &mut RlpStream) {
        match self {
            Node::Empty => {
                stream.append_empty_data();
            }
            Node::Leaf(path, value) => {
                stream.begin_list(2);
                stream.append(path);
                stream.append(value);
            }
            Node::Extension(path, child) => {
                stream.begin_list(2);
                stream.append(path);
                stream.append(&child.as_ref());
            }
            Node::Branch(children, value) => {
                stream.begin_list(17);
                for child in children.iter() {
                    match child {
                        Some(hash) => stream.append(&hash.as_ref()),
                        None => stream.append_empty_data(),
                    };
                }
                match value {
                    Some(v) => stream.append(v),
                    None => stream.append_empty_data(),
                };
            }
        }
    }
}

impl Decodable for Node {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        if rlp.is_empty() {
            return Ok(Node::Empty);
        }
        
        let item_count = rlp.item_count()?;
        
        match item_count {
            2 => {
                let path: Vec<u8> = rlp.val_at(0)?;
                let (_, is_leaf) = compact_decode(&path);
                
                if is_leaf {
                    let value: Vec<u8> = rlp.val_at(1)?;
                    Ok(Node::Leaf(path, value))
                } else {
                    let child: Vec<u8> = rlp.val_at(1)?;
                    if child.len() != 32 {
                        return Err(DecoderError::Custom("Invalid hash length"));
                    }
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&child);
                    Ok(Node::Extension(path, hash))
                }
            }
            17 => {
                let mut children = Box::new([None; 16]);
                for i in 0..16 {
                    let child_data: Vec<u8> = rlp.val_at(i)?;
                    if !child_data.is_empty() {
                        if child_data.len() != 32 {
                            return Err(DecoderError::Custom("Invalid hash length in branch"));
                        }
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&child_data);
                        children[i] = Some(hash);
                    }
                }
                
                let value_data: Vec<u8> = rlp.val_at(16)?;
                let value = if value_data.is_empty() {
                    None
                } else {
                    Some(value_data)
                };
                
                Ok(Node::Branch(children, value))
            }
            _ => Err(DecoderError::RlpIncorrectListLen),
        }
    }
}

/// Computes Keccak-256 hash of the input
pub fn keccak256(data: &[u8]) -> Hash {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_encoding() {
        let node = Node::new_leaf(&[1, 2, 3, 4], b"value".to_vec());
        let encoded = node.encode_raw();
        let decoded = Node::decode_raw(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_extension_encoding() {
        let hash = [0u8; 32];
        let node = Node::new_extension(&[1, 2, 3, 4], hash);
        let encoded = node.encode_raw();
        let decoded = Node::decode_raw(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_branch_encoding() {
        let mut node = Node::new_branch();
        if let Node::Branch(ref mut children, ref mut value) = node {
            children[0] = Some([1u8; 32]);
            children[15] = Some([2u8; 32]);
            *value = Some(b"branch_value".to_vec());
        }
        let encoded = node.encode_raw();
        let decoded = Node::decode_raw(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_empty_node() {
        let node = Node::Empty;
        let encoded = node.encode_raw();
        let decoded = Node::decode_raw(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash = keccak256(data);
        assert_eq!(hash.len(), 32);
    }
}

