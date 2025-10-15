/// Utility functions for working with nibbles (4-bit values) in Merkle Patricia Tries.
/// In Ethereum's MPT, keys are represented as sequences of nibbles (hex digits).

/// Converts a byte slice into a vector of nibbles (each byte becomes 2 nibbles)
pub fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        nibbles.push(byte >> 4);      // high nibble
        nibbles.push(byte & 0x0F);    // low nibble
    }
    nibbles
}

/// Converts a vector of nibbles back to bytes
/// Note: nibbles.len() must be even
pub fn nibbles_to_bytes(nibbles: &[u8]) -> Vec<u8> {
    assert!(nibbles.len() % 2 == 0, "Nibbles length must be even");
    let mut bytes = Vec::with_capacity(nibbles.len() / 2);
    for chunk in nibbles.chunks(2) {
        bytes.push((chunk[0] << 4) | chunk[1]);
    }
    bytes
}

/// Compact encoding for nibble paths as used in Ethereum MPT.
/// 
/// The compact encoding combines the terminator flag with the path:
/// - If the path has an odd length, the first nibble stores the flag
/// - If the path has an even length, we add a padding nibble
/// 
/// hex char    bits    |    node type partial     path length
/// ----------------------------------------------------------
///   0        0000     |       extension              even
///   1        0001     |       extension              odd
///   2        0010     |   terminating (leaf)         even
///   3        0011     |   terminating (leaf)         odd
pub fn compact_encode(nibbles: &[u8], is_leaf: bool) -> Vec<u8> {
    let mut flags = if is_leaf { 2u8 } else { 0u8 };
    let mut encoded = nibbles.to_vec();
    
    if nibbles.len() % 2 == 1 {
        // Odd length: set the odd flag and prepend the flag nibble
        flags |= 1;
        encoded.insert(0, flags);
    } else {
        // Even length: prepend two nibbles (flag and padding)
        encoded.insert(0, 0);
        encoded.insert(0, flags);
    }
    
    nibbles_to_bytes(&encoded)
}

/// Decodes a compact-encoded path back to nibbles and determines if it's a leaf
pub fn compact_decode(encoded: &[u8]) -> (Vec<u8>, bool) {
    let nibbles = bytes_to_nibbles(encoded);
    let first_nibble = nibbles[0];
    
    let is_leaf = (first_nibble & 2) == 2;
    let is_odd = (first_nibble & 1) == 1;
    
    let result = if is_odd {
        nibbles[1..].to_vec()
    } else {
        nibbles[2..].to_vec()
    };
    
    (result, is_leaf)
}

/// Finds the common prefix length between two nibble slices
pub fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b.iter())
        .take_while(|(x, y)| x == y)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_nibbles() {
        assert_eq!(bytes_to_nibbles(&[0xAB, 0xCD]), vec![10, 11, 12, 13]);
        assert_eq!(bytes_to_nibbles(&[0x12, 0x34]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_nibbles_to_bytes() {
        assert_eq!(nibbles_to_bytes(&[10, 11, 12, 13]), vec![0xAB, 0xCD]);
        assert_eq!(nibbles_to_bytes(&[1, 2, 3, 4]), vec![0x12, 0x34]);
    }

    #[test]
    fn test_compact_encode_leaf_even() {
        let nibbles = vec![1, 2, 3, 4];
        let encoded = compact_encode(&nibbles, true);
        assert_eq!(encoded, vec![0x20, 0x12, 0x34]);
    }

    #[test]
    fn test_compact_encode_leaf_odd() {
        let nibbles = vec![1, 2, 3];
        let encoded = compact_encode(&nibbles, true);
        assert_eq!(encoded, vec![0x31, 0x23]);
    }

    #[test]
    fn test_compact_encode_extension_even() {
        let nibbles = vec![1, 2, 3, 4];
        let encoded = compact_encode(&nibbles, false);
        assert_eq!(encoded, vec![0x00, 0x12, 0x34]);
    }

    #[test]
    fn test_compact_encode_extension_odd() {
        let nibbles = vec![1, 2, 3];
        let encoded = compact_encode(&nibbles, false);
        assert_eq!(encoded, vec![0x11, 0x23]);
    }

    #[test]
    fn test_compact_decode() {
        let (nibbles, is_leaf) = compact_decode(&[0x20, 0x12, 0x34]);
        assert_eq!(nibbles, vec![1, 2, 3, 4]);
        assert!(is_leaf);

        let (nibbles, is_leaf) = compact_decode(&[0x00, 0x12, 0x34]);
        assert_eq!(nibbles, vec![1, 2, 3, 4]);
        assert!(!is_leaf);
    }

    #[test]
    fn test_common_prefix_len() {
        assert_eq!(common_prefix_len(&[1, 2, 3], &[1, 2, 4]), 2);
        assert_eq!(common_prefix_len(&[1, 2, 3], &[1, 2, 3]), 3);
        assert_eq!(common_prefix_len(&[1, 2, 3], &[4, 5, 6]), 0);
    }
}

