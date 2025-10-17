use mpt::MerklePatriciaTrie;

fn main() {
    println!("=== Merkle Patricia Trie Demo ===\n");

    // Create a new trie
    let mut trie = MerklePatriciaTrie::new();
    println!("1. Created empty trie");
    println!("   Root hash: {}\n", hex::encode(trie.root_hash()));

    // Insert some values
    println!("2. Inserting key-value pairs:");
    trie.insert(b"do", b"verb".to_vec());
    println!("   Inserted: 'do' -> 'verb'");
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    
    trie.insert(b"dog", b"puppy".to_vec());
    println!("   Inserted: 'dog' -> 'puppy'");
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    
    trie.insert(b"doge", b"coin".to_vec());
    println!("   Inserted: 'doge' -> 'coin'");
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    
    trie.insert(b"horse", b"stallion".to_vec());
    println!("   Inserted: 'horse' -> 'stallion'");
    println!("   Root hash: {}\n", hex::encode(trie.root_hash()));

    // Retrieve values
    println!("3. Retrieving values:");
    println!("   'do' -> {:?}", String::from_utf8_lossy(&trie.get(b"do").unwrap()));
    println!("   'dog' -> {:?}", String::from_utf8_lossy(&trie.get(b"dog").unwrap()));
    println!("   'doge' -> {:?}", String::from_utf8_lossy(&trie.get(b"doge").unwrap()));
    println!("   'horse' -> {:?}", String::from_utf8_lossy(&trie.get(b"horse").unwrap()));
    println!("   'cat' -> {:?}\n", trie.get(b"cat"));

    // Update a value
    println!("4. Updating a value:");
    trie.insert(b"dog", b"animal".to_vec());
    println!("   Updated: 'dog' -> 'animal'");
    println!("   'dog' -> {:?}", String::from_utf8_lossy(&trie.get(b"dog").unwrap()));
    println!("   Root hash: {}\n", hex::encode(trie.root_hash()));

    // Delete a value
    println!("5. Deleting a value:");
    trie.delete(b"dog");
    println!("   Deleted: 'dog'");
    println!("   'dog' -> {:?}", trie.get(b"dog"));
    println!("   Root hash: {}\n", hex::encode(trie.root_hash()));

    // Verify other values are still present
    println!("6. Verifying remaining values:");
    println!("   'do' -> {:?}", String::from_utf8_lossy(&trie.get(b"do").unwrap()));
    println!("   'doge' -> {:?}", String::from_utf8_lossy(&trie.get(b"doge").unwrap()));
    println!("   'horse' -> {:?}", String::from_utf8_lossy(&trie.get(b"horse").unwrap()));

    println!("\n=== Demo Complete ===");
}

#[cfg(test)]
mod tests {
    use mpt::MerklePatriciaTrie;

    #[test]
    fn test_basic_operations() {
        let mut trie: MerklePatriciaTrie = MerklePatriciaTrie::new();
        
        // Test insert and get
        trie.insert(b"key1", b"value1".to_vec());
        assert_eq!(trie.get(b"key1"), Some(b"value1".to_vec()));
        
        // Test non-existent key
        assert_eq!(trie.get(b"key2"), None);
    }

    #[test]
    fn test_update_value() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"key", b"value1".to_vec());
        assert_eq!(trie.get(b"key"), Some(b"value1".to_vec()));
        
        trie.insert(b"key", b"value2".to_vec());
        assert_eq!(trie.get(b"key"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_multiple_keys_with_common_prefix() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"do", b"verb".to_vec());
        trie.insert(b"dog", b"puppy".to_vec());
        trie.insert(b"doge", b"coin".to_vec());
        trie.insert(b"dodge", b"avoid".to_vec());
        
        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
        assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
        assert_eq!(trie.get(b"doge"), Some(b"coin".to_vec()));
        assert_eq!(trie.get(b"dodge"), Some(b"avoid".to_vec()));
    }

    #[test]
    fn test_delete_single_key() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"key", b"value".to_vec());
        assert_eq!(trie.get(b"key"), Some(b"value".to_vec()));
        
        trie.delete(b"key");
        assert_eq!(trie.get(b"key"), None);
    }

    #[test]
    fn test_delete_with_multiple_keys() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"do", b"verb".to_vec());
        trie.insert(b"dog", b"puppy".to_vec());
        trie.insert(b"doge", b"coin".to_vec());
        
        trie.delete(b"dog");
        
        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
        assert_eq!(trie.get(b"dog"), None);
        assert_eq!(trie.get(b"doge"), Some(b"coin".to_vec()));
    }

    #[test]
    fn test_root_hash_consistency() {
        let mut trie1 = MerklePatriciaTrie::new();
        let mut trie2 = MerklePatriciaTrie::new();
        
        // Insert in same order
        trie1.insert(b"do", b"verb".to_vec());
        trie1.insert(b"dog", b"puppy".to_vec());
        
        trie2.insert(b"do", b"verb".to_vec());
        trie2.insert(b"dog", b"puppy".to_vec());
        
        // Root hashes should match
        assert_eq!(trie1.root_hash(), trie2.root_hash());
    }

    #[test]
    fn test_root_hash_differs_for_different_content() {
        let mut trie1 = MerklePatriciaTrie::new();
        let mut trie2 = MerklePatriciaTrie::new();
        
        trie1.insert(b"key", b"value1".to_vec());
        trie2.insert(b"key", b"value2".to_vec());
        
        assert_ne!(trie1.root_hash(), trie2.root_hash());
    }

    #[test]
    fn test_empty_trie_operations() {
        let mut trie = MerklePatriciaTrie::new();
        
        assert_eq!(trie.get(b"any"), None);
        
        trie.delete(b"nonexistent");
        assert_eq!(trie.get(b"nonexistent"), None);
    }

    #[test]
    fn test_delete_and_reinsert() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"key", b"value1".to_vec());
        let hash1 = trie.root_hash();
        
        trie.delete(b"key");
        trie.insert(b"key", b"value1".to_vec());
        let hash2 = trie.root_hash();
        
        assert_eq!(hash1, hash2);
        assert_eq!(trie.get(b"key"), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_many_keys() {
        let mut trie = MerklePatriciaTrie::new();
        
        for i in 0..100 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            trie.insert(key.as_bytes(), value.as_bytes().to_vec());
        }
        
        for i in 0..100 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            assert_eq!(trie.get(key.as_bytes()), Some(value.as_bytes().to_vec()));
        }
    }

    #[test]
    fn test_different_key_lengths() {
        let mut trie = MerklePatriciaTrie::new();
        
        trie.insert(b"a", b"1".to_vec());
        trie.insert(b"ab", b"2".to_vec());
        trie.insert(b"abc", b"3".to_vec());
        trie.insert(b"abcd", b"4".to_vec());
        
        assert_eq!(trie.get(b"a"), Some(b"1".to_vec()));
        assert_eq!(trie.get(b"ab"), Some(b"2".to_vec()));
        assert_eq!(trie.get(b"abc"), Some(b"3".to_vec()));
        assert_eq!(trie.get(b"abcd"), Some(b"4".to_vec()));
    }
}
