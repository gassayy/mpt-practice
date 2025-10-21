/// Demonstration of how data is stored in the MPT
/// Run with: cargo run --example storage_demo

use mpt::MerklePatriciaTrie;

fn main() {
    println!("=== MPT Storage Demonstration ===\n");

    let mut trie = MerklePatriciaTrie::new();
    
    println!("1. Empty Trie");
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    println!("   (This is hash of empty bytes)\n");

    // Insert first key
    println!("2. Insert 'dog' → 'puppy'");
    trie.insert(b"dog", b"puppy".to_vec());
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    println!("   Tree structure: Single Leaf Node");
    println!("   - Path: compact([6,4,6,15,6,7]) = nibbles of 'dog'");
    println!("   - Value: 'puppy' (stored as raw bytes)\n");

    // Insert related key
    println!("3. Insert 'doge' → 'coin'");
    trie.insert(b"doge", b"coin".to_vec());
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    println!("   Tree structure: Extension → Branch → ...");
    println!("   - Extension compresses shared 'do' prefix");
    println!("   - Branch splits on 'g' vs 'ge'\n");

    // Verification
    println!("4. Verify Keys Exist");
    match trie.get(b"dog") {
        Some(v) => println!("   ✓ 'dog' found: '{}'", String::from_utf8_lossy(&v)),
        None => println!("   ✗ 'dog' not found"),
    }
    match trie.get(b"doge") {
        Some(v) => println!("   ✓ 'doge' found: '{}'", String::from_utf8_lossy(&v)),
        None => println!("   ✗ 'doge' not found"),
    }
    match trie.get(b"cat") {
        Some(v) => println!("   ✓ 'cat' found: '{}'", String::from_utf8_lossy(&v)),
        None => println!("   ✗ 'cat' not found"),
    }
    println!();

    // Show how storage works internally
    println!("5. Storage Internals");
    println!("   Storage is: HashMap<Hash, Node>");
    println!("   - Keys are 32-byte hashes of RLP-encoded nodes");
    println!("   - Values are the actual Node structures");
    println!("   - Root hash points to the top node\n");

    // Add unrelated key
    println!("6. Insert 'horse' → 'stallion'");
    trie.insert(b"horse", b"stallion".to_vec());
    println!("   Root hash: {}", hex::encode(trie.root_hash()));
    println!("   Tree now has two branches:");
    println!("   - One for 'do*' keys (dog, doge)");
    println!("   - One for 'horse'\n");

    // Show root hash changes with any modification
    println!("7. Root Hash is Cryptographic Commitment");
    let root_before = trie.root_hash();
    trie.insert(b"dog", b"animal".to_vec());  // Update value
    let root_after = trie.root_hash();
    println!("   Root before update: {}", hex::encode(root_before));
    println!("   Root after update:  {}", hex::encode(root_after));
    println!("   Different! Any change → new root hash\n");

    // Demonstrate verification without full tree
    println!("8. Light Client Verification (Concept)");
    println!("   Problem: How to verify 'doge' → 'coin' without full trie?");
    println!("   Solution: Merkle Proof");
    println!("   ");
    println!("   What you need:");
    println!("   - Trusted root hash: {}", hex::encode(trie.root_hash())[..16].to_string() + "...");
    println!("   - Proof: [Extension node, Branch node, Leaf node]");
    println!("   - Only ~3-5 nodes instead of entire trie!");
    println!("   ");
    println!("   Verification:");
    println!("   1. Hash the leaf node → Hash₃");
    println!("   2. Verify Hash₃ is in branch node");
    println!("   3. Hash the branch → Hash₂");
    println!("   4. Verify Hash₂ is in extension node");
    println!("   5. Hash the extension → Hash₁");
    println!("   6. Verify Hash₁ == trusted root hash ✓");
    println!();

    println!("=== Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("• Data stored as-is in leaf nodes (not pre-hashed)");
    println!("• Nodes stored by their hash (content-addressed)");
    println!("• Root hash commits to entire tree state");
    println!("• Can verify specific keys with small proofs");
    
    println!("\n\n");
    println!("═══════════════════════════════════════════════════════════════");
    println!("VISUALIZATION: Tree Structure");
    println!("═══════════════════════════════════════════════════════════════");
    trie.print_tree();
    
    println!("\n\n");
    println!("═══════════════════════════════════════════════════════════════");
    println!("VISUALIZATION: Storage Details");
    println!("═══════════════════════════════════════════════════════════════");
    trie.print_storage();
}

