/// Simple demonstration of MPT pretty printing
/// Run with: cargo run --example print_demo

use mpt::MerklePatriciaTrie;

fn main() {
    println!("🌳 Merkle Patricia Trie Pretty Printing Demo\n");

    // Create a new trie
    let mut trie = MerklePatriciaTrie::new();
    
    // Example 1: Empty trie
    println!("══════════════════════════════════════════════════════════════");
    println!("Example 1: Empty Trie");
    println!("══════════════════════════════════════════════════════════════\n");
    trie.print_tree();
    
    // Example 2: Single leaf
    println!("\n\n══════════════════════════════════════════════════════════════");
    println!("Example 2: Single Leaf");
    println!("══════════════════════════════════════════════════════════════\n");
    trie.insert(b"hello", b"world".to_vec());
    trie.print_tree();
    
    // Example 3: Two related keys (creates branch)
    println!("\n\n══════════════════════════════════════════════════════════════");
    println!("Example 3: Two Keys with Common Prefix");
    println!("══════════════════════════════════════════════════════════════\n");
    trie.insert(b"help", b"me".to_vec());
    trie.print_tree();
    
    // Example 4: Classic example (dog, doge, do)
    println!("\n\n══════════════════════════════════════════════════════════════");
    println!("Example 4: Classic 'dog' Example");
    println!("══════════════════════════════════════════════════════════════\n");
    let mut trie2 = MerklePatriciaTrie::new();
    trie2.insert(b"do", b"verb".to_vec());
    trie2.insert(b"dog", b"puppy".to_vec());
    trie2.insert(b"doge", b"coin".to_vec());
    trie2.insert(b"horse", b"stallion".to_vec());
    
    println!("Tree structure:");
    trie2.print_tree();
    
    println!("\n\nStorage contents:");
    trie2.print_storage();
    
    // Example 5: More complex tree
    println!("\n\n══════════════════════════════════════════════════════════════");
    println!("Example 5: Complex Tree with Multiple Branches");
    println!("══════════════════════════════════════════════════════════════\n");
    let mut trie3 = MerklePatriciaTrie::new();
    
    // Different first letters
    trie3.insert(b"apple", b"fruit".to_vec());
    trie3.insert(b"banana", b"yellow".to_vec());
    trie3.insert(b"cherry", b"red".to_vec());
    
    // Same first letter
    trie3.insert(b"avocado", b"green".to_vec());
    trie3.insert(b"apricot", b"orange".to_vec());
    
    trie3.print_tree();
    
    println!("\n\n✅ Demo complete! The functions are:");
    println!("   • trie.print_tree()    - Shows tree structure");
    println!("   • trie.print_storage() - Shows all nodes in storage");
}

