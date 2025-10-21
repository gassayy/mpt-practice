use crate::node::{Node, Hash, keccak256};
use crate::nibbles::{bytes_to_nibbles, compact_decode, common_prefix_len};
use std::collections::HashMap;

/// The Merkle Patricia Trie structure
pub struct MerklePatriciaTrie {
    /// Storage for nodes, indexed by their hash
    storage: HashMap<Hash, Node>,
    /// The root hash of the trie
    root: Hash,
}

impl MerklePatriciaTrie {
    /// Creates a new empty trie
    pub fn new() -> Self {
        let empty_hash = keccak256(&[]);
        Self {
            storage: HashMap::new(),
            root: empty_hash,
        }
    }
    
    /// Returns the root hash of the trie
    pub fn root_hash(&self) -> Hash {
        self.root
    }
    
    /// Inserts a key-value pair into the trie
    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let nibbles = bytes_to_nibbles(key);
        self.root = self.insert_at(&nibbles, value, self.root);
    }
    
    /// Retrieves a value by key from the trie
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let nibbles = bytes_to_nibbles(key);
        self.get_at(&nibbles, self.root)
    }
    
    /// Deletes a key from the trie
    pub fn delete(&mut self, key: &[u8]) {
        let nibbles = bytes_to_nibbles(key);
        self.root = self.delete_at(&nibbles, self.root);
    }
    
    /// Helper: inserts a value at a specific node
    fn insert_at(&mut self, path: &[u8], value: Vec<u8>, node_hash: Hash) -> Hash {
        // Get the node or treat as empty
        let node = self.get_node(node_hash).unwrap_or(Node::Empty);
        
        match node {
            Node::Empty => {
                // Create a new leaf node
                let leaf = Node::new_leaf(path, value);
                self.store_node(leaf)
            }
            
            Node::Leaf(encoded_path, old_value) => {
                let (leaf_path, _) = compact_decode(&encoded_path);
                
                if leaf_path == path {
                    // Same key, update value
                    let leaf = Node::new_leaf(path, value);
                    self.store_node(leaf)
                } else {
                    // Split the leaf into a branch
                    let common_len = common_prefix_len(&leaf_path, path);
                    
                    let new_branch_hash = self.create_branch_from_divergence(
                        &leaf_path[common_len..],
                        old_value,
                        &path[common_len..],
                        value,
                    );
                    
                    if common_len > 0 {
                        // Create an extension node
                        let ext = Node::new_extension(&path[..common_len], new_branch_hash);
                        self.store_node(ext)
                    } else {
                        new_branch_hash
                    }
                }
            }
            
            Node::Extension(encoded_path, child_hash) => {
                let (ext_path, _) = compact_decode(&encoded_path);
                let common_len = common_prefix_len(&ext_path, path);
                
                if common_len == ext_path.len() {
                    // Continue down the extension
                    let new_child = self.insert_at(&path[common_len..], value, child_hash);
                    let ext = Node::new_extension(&ext_path, new_child);
                    self.store_node(ext)
                } else {
                    // Split the extension
                    // Need to handle the old child properly
                    let remaining_ext_path = &ext_path[common_len..];
                    let mut branch = Node::new_branch();
                    
                    if remaining_ext_path.len() == 1 {
                        // Direct child
                        if let Node::Branch(ref mut children, _) = branch {
                            children[remaining_ext_path[0] as usize] = Some(child_hash);
                        }
                    } else {
                        // Need extension
                        let ext = Node::new_extension(&remaining_ext_path[1..], child_hash);
                        let ext_hash = self.store_node(ext);
                        if let Node::Branch(ref mut children, _) = branch {
                            children[remaining_ext_path[0] as usize] = Some(ext_hash);
                        }
                    }
                    
                    // Insert new value
                    let remaining_new_path = &path[common_len..];
                    if remaining_new_path.is_empty() {
                        if let Node::Branch(_, ref mut branch_value) = branch {
                            *branch_value = Some(value);
                        }
                    } else if remaining_new_path.len() == 1 {
                        let leaf = Node::new_leaf(&[], value);
                        let leaf_hash = self.store_node(leaf);
                        if let Node::Branch(ref mut children, _) = branch {
                            children[remaining_new_path[0] as usize] = Some(leaf_hash);
                        }
                    } else {
                        let leaf = Node::new_leaf(&remaining_new_path[1..], value);
                        let leaf_hash = self.store_node(leaf);
                        if let Node::Branch(ref mut children, _) = branch {
                            children[remaining_new_path[0] as usize] = Some(leaf_hash);
                        }
                    }
                    
                    let branch_hash = self.store_node(branch);
                    
                    if common_len > 0 {
                        let ext = Node::new_extension(&path[..common_len], branch_hash);
                        self.store_node(ext)
                    } else {
                        branch_hash
                    }
                }
            }
            
            Node::Branch(mut children, mut branch_value) => {
                if path.is_empty() {
                    // Insert value at this branch
                    branch_value = Some(value);
                    let branch = Node::Branch(children, branch_value);
                    self.store_node(branch)
                } else {
                    let idx = path[0] as usize;
                    let child_hash = children[idx].unwrap_or_else(|| keccak256(&[]));
                    let new_child = self.insert_at(&path[1..], value, child_hash);
                    children[idx] = Some(new_child);
                    let branch = Node::Branch(children, branch_value);
                    self.store_node(branch)
                }
            }
        }
    }
    
    /// Helper: creates a branch from two diverging paths
    fn create_branch_from_divergence(
        &mut self,
        path1: &[u8],
        value1: Vec<u8>,
        path2: &[u8],
        value2: Vec<u8>,
    ) -> Hash {
        let mut branch = Node::new_branch();
        
        // Handle first path
        if path1.is_empty() {
            if let Node::Branch(_, ref mut branch_value) = branch {
                *branch_value = Some(value1);
            }
        } else {
            let leaf1 = if path1.len() == 1 {
                Node::new_leaf(&[], value1)
            } else {
                Node::new_leaf(&path1[1..], value1)
            };
            let leaf1_hash = self.store_node(leaf1);
            if let Node::Branch(ref mut children, _) = branch {
                children[path1[0] as usize] = Some(leaf1_hash);
            }
        }
        
        // Handle second path
        if path2.is_empty() {
            if let Node::Branch(_, ref mut branch_value) = branch {
                *branch_value = Some(value2);
            }
        } else {
            let leaf2 = if path2.len() == 1 {
                Node::new_leaf(&[], value2)
            } else {
                Node::new_leaf(&path2[1..], value2)
            };
            let leaf2_hash = self.store_node(leaf2);
            if let Node::Branch(ref mut children, _) = branch {
                children[path2[0] as usize] = Some(leaf2_hash);
            }
        }
        
        self.store_node(branch)
    }
    
    /// Helper: retrieves a value at a specific node
    fn get_at(&self, path: &[u8], node_hash: Hash) -> Option<Vec<u8>> {
        let node = self.get_node(node_hash)?;
        
        match node {
            Node::Empty => None,
            
            Node::Leaf(encoded_path, value) => {
                let (leaf_path, _) = compact_decode(&encoded_path);
                if leaf_path == path {
                    Some(value)
                } else {
                    None
                }
            }
            
            Node::Extension(encoded_path, child_hash) => {
                let (ext_path, _) = compact_decode(&encoded_path);
                if path.len() < ext_path.len() || &path[..ext_path.len()] != ext_path.as_slice() {
                    None
                } else {
                    self.get_at(&path[ext_path.len()..], child_hash)
                }
            }
            
            Node::Branch(children, branch_value) => {
                if path.is_empty() {
                    branch_value
                } else {
                    let idx = path[0] as usize;
                    children[idx].and_then(|child_hash| self.get_at(&path[1..], child_hash))
                }
            }
        }
    }
    
    /// Helper: deletes a key at a specific node
    fn delete_at(&mut self, path: &[u8], node_hash: Hash) -> Hash {
        let node = match self.get_node(node_hash) {
            Some(n) => n,
            None => return keccak256(&[]), // Empty node
        };
        
        match node {
            Node::Empty => keccak256(&[]),
            
            Node::Leaf(encoded_path, _) => {
                let (leaf_path, _) = compact_decode(&encoded_path);
                if leaf_path == path {
                    // Delete this leaf
                    keccak256(&[])
                } else {
                    // Key not found, keep the leaf
                    node_hash
                }
            }
            
            Node::Extension(encoded_path, child_hash) => {
                let (ext_path, _) = compact_decode(&encoded_path);
                if path.len() < ext_path.len() || &path[..ext_path.len()] != ext_path.as_slice() {
                    // Path doesn't match, keep the extension
                    node_hash
                } else {
                    let new_child = self.delete_at(&path[ext_path.len()..], child_hash);
                    let empty_hash = keccak256(&[]);
                    
                    if new_child == empty_hash {
                        // Child was deleted
                        empty_hash
                    } else {
                        // Update extension
                        let ext = Node::new_extension(&ext_path, new_child);
                        self.store_node(ext)
                    }
                }
            }
            
            Node::Branch(mut children, branch_value) => {
                if path.is_empty() {
                    // Delete value at branch
                    let branch = Node::Branch(children, None);
                    self.normalize_branch(branch)
                } else {
                    let idx = path[0] as usize;
                    if let Some(child_hash) = children[idx] {
                        let new_child = self.delete_at(&path[1..], child_hash);
                        let empty_hash = keccak256(&[]);
                        
                        if new_child == empty_hash {
                            children[idx] = None;
                        } else {
                            children[idx] = Some(new_child);
                        }
                    }
                    
                    let branch = Node::Branch(children, branch_value);
                    self.normalize_branch(branch)
                }
            }
        }
    }
    
    /// Helper: normalizes a branch node (converts to simpler form if possible)
    fn normalize_branch(&mut self, node: Node) -> Hash {
        if let Node::Branch(children, branch_value) = node {
            let child_count: usize = children.iter().filter(|c| c.is_some()).count();
            
            if child_count == 0 && branch_value.is_none() {
                // Empty branch
                return keccak256(&[]);
            }
            
            if child_count == 1 && branch_value.is_none() {
                // Single child, convert to extension or return child
                let (idx, child_hash) = children
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.is_some())
                    .map(|(i, c)| (i, c.unwrap()))
                    .unwrap();
                
                // Try to merge with child if it's an extension or leaf
                if let Some(child_node) = self.get_node(child_hash) {
                    match child_node {
                        Node::Extension(encoded_path, grandchild_hash) => {
                            let (ext_path, _) = compact_decode(&encoded_path);
                            let mut new_path = vec![idx as u8];
                            new_path.extend_from_slice(&ext_path);
                            let ext = Node::new_extension(&new_path, grandchild_hash);
                            return self.store_node(ext);
                        }
                        Node::Leaf(encoded_path, value) => {
                            let (leaf_path, _) = compact_decode(&encoded_path);
                            let mut new_path = vec![idx as u8];
                            new_path.extend_from_slice(&leaf_path);
                            let leaf = Node::new_leaf(&new_path, value);
                            return self.store_node(leaf);
                        }
                        _ => {}
                    }
                }
                
                // Just create an extension to the child
                let ext = Node::new_extension(&[idx as u8], child_hash);
                return self.store_node(ext);
            }
            
            // Keep as branch
            let branch = Node::Branch(children, branch_value);
            self.store_node(branch)
        } else {
            self.store_node(node)
        }
    }
    
    /// Stores a node and returns its hash
    fn store_node(&mut self, node: Node) -> Hash {
        let hash = node.hash();
        self.storage.insert(hash, node);
        hash
    }
    
    /// Retrieves a node by hash
    fn get_node(&self, hash: Hash) -> Option<Node> {
        let empty_hash = keccak256(&[]);
        if hash == empty_hash {
            return Some(Node::Empty);
        }
        self.storage.get(&hash).cloned()
    }
}

impl Default for MerklePatriciaTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl MerklePatriciaTrie {
    /// Pretty prints the entire trie structure as a tree
    pub fn print_tree(&self) {
        println!("╔═══════════════════════════════════════════════════════════════");
        println!("║ Merkle Patricia Trie Structure");
        println!("╠═══════════════════════════════════════════════════════════════");
        println!("║ Root Hash: 0x{}", hex_truncated(&self.root));
        println!("╚═══════════════════════════════════════════════════════════════\n");
        
        let empty_hash = keccak256(&[]);
        if self.root == empty_hash {
            println!("  (empty trie)");
            return;
        }
        
        self.print_node(self.root, "", true, "");
    }
    
    /// Helper function to recursively print a node and its children
    fn print_node(&self, node_hash: Hash, prefix: &str, is_last: bool, path_so_far: &str) {
        let node = match self.get_node(node_hash) {
            Some(n) => n,
            None => {
                println!("{}{}── [MISSING NODE]", prefix, if is_last { "└" } else { "├" });
                return;
            }
        };
        
        let branch = if is_last { "└──" } else { "├──" };
        let extension = if is_last { "    " } else { "│   " };
        
        match node {
            Node::Empty => {
                println!("{}{} Empty", prefix, branch);
            }
            
            Node::Leaf(encoded_path, value) => {
                let (nibbles, _) = compact_decode(&encoded_path);
                let full_path = format!("{}{}", path_so_far, nibbles_to_hex(&nibbles));
                let value_str = format_value(&value);
                println!("{}{} Leaf", prefix, branch);
                println!("{}{}   Path: {} → {}", prefix, extension, full_path, nibbles_to_hex(&nibbles));
                println!("{}{}   Value: {}", prefix, extension, value_str);
                println!("{}{}   Hash: 0x{}", prefix, extension, hex_truncated(&node_hash));
            }
            
            Node::Extension(encoded_path, child_hash) => {
                let (nibbles, _) = compact_decode(&encoded_path);
                let new_path = format!("{}{}", path_so_far, nibbles_to_hex(&nibbles));
                println!("{}{} Extension", prefix, branch);
                println!("{}{}   Path: {}", prefix, extension, nibbles_to_hex(&nibbles));
                println!("{}{}   Hash: 0x{}", prefix, extension, hex_truncated(&node_hash));
                
                let new_prefix = format!("{}{}", prefix, extension);
                self.print_node(child_hash, &new_prefix, true, &new_path);
            }
            
            Node::Branch(children, branch_value) => {
                println!("{}{} Branch", prefix, branch);
                if let Some(val) = branch_value {
                    println!("{}{}   Value: {}", prefix, extension, format_value(&val));
                }
                println!("{}{}   Hash: 0x{}", prefix, extension, hex_truncated(&node_hash));
                
                // Count non-empty children
                let non_empty: Vec<(usize, Hash)> = children
                    .iter()
                    .enumerate()
                    .filter_map(|(i, h)| h.map(|hash| (i, hash)))
                    .collect();
                
                for (idx, (nibble, child_hash)) in non_empty.iter().enumerate() {
                    let is_last_child = idx == non_empty.len() - 1;
                    let new_path = format!("{}{:x}", path_so_far, nibble);
                    let new_prefix = format!("{}{}   ", prefix, extension);
                    
                    println!("{}{}[{:x}]", new_prefix, if is_last_child { "└" } else { "├" }, nibble);
                    let child_prefix = format!("{}{}   ", new_prefix, if is_last_child { " " } else { "│" });
                    self.print_node(*child_hash, &child_prefix, true, &new_path);
                }
            }
        }
    }
    
    /// Prints all nodes in storage with their details
    pub fn print_storage(&self) {
        println!("╔═══════════════════════════════════════════════════════════════");
        println!("║ Trie Storage Contents");
        println!("╠═══════════════════════════════════════════════════════════════");
        println!("║ Total nodes: {}", self.storage.len());
        println!("║ Root hash: 0x{}", hex_truncated(&self.root));
        println!("╚═══════════════════════════════════════════════════════════════\n");
        
        if self.storage.is_empty() {
            println!("  (no nodes in storage)");
            return;
        }
        
        for (idx, (hash, node)) in self.storage.iter().enumerate() {
            println!("Node #{}", idx + 1);
            println!("  Hash: 0x{}", hex_full(hash));
            
            match node {
                Node::Empty => {
                    println!("  Type: Empty");
                }
                Node::Leaf(encoded_path, value) => {
                    let (nibbles, _) = compact_decode(encoded_path);
                    println!("  Type: Leaf");
                    println!("  Path (nibbles): {}", nibbles_to_hex(&nibbles));
                    println!("  Path (encoded): 0x{}", hex_bytes(encoded_path));
                    println!("  Value: {}", format_value(value));
                }
                Node::Extension(encoded_path, child_hash) => {
                    let (nibbles, _) = compact_decode(encoded_path);
                    println!("  Type: Extension");
                    println!("  Path (nibbles): {}", nibbles_to_hex(&nibbles));
                    println!("  Path (encoded): 0x{}", hex_bytes(encoded_path));
                    println!("  Child: 0x{}", hex_truncated(child_hash));
                }
                Node::Branch(children, branch_value) => {
                    println!("  Type: Branch");
                    if let Some(val) = branch_value {
                        println!("  Branch value: {}", format_value(val));
                    }
                    let non_empty: Vec<usize> = children
                        .iter()
                        .enumerate()
                        .filter_map(|(i, h)| if h.is_some() { Some(i) } else { None })
                        .collect();
                    println!("  Children: [{}]", 
                        non_empty.iter()
                            .map(|i| format!("{:x}", i))
                            .collect::<Vec<_>>()
                            .join(", "));
                    
                    for (i, child_hash) in children.iter().enumerate() {
                        if let Some(hash) = child_hash {
                            println!("    [{:x}] → 0x{}", i, hex_truncated(hash));
                        }
                    }
                }
            }
            println!();
        }
    }
}

// Helper functions for formatting

/// Converts nibbles to hex string representation
fn nibbles_to_hex(nibbles: &[u8]) -> String {
    if nibbles.is_empty() {
        return String::from("(empty)");
    }
    nibbles.iter()
        .map(|n| format!("{:x}", n))
        .collect::<String>()
}

/// Formats a value for display (shows as string if printable, otherwise as hex)
fn format_value(value: &[u8]) -> String {
    // Try to display as UTF-8 string if it's valid
    if let Ok(s) = std::str::from_utf8(value) {
        if s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
            return format!("\"{}\"", s);
        }
    }
    // Otherwise show as hex
    format!("0x{}", hex_bytes(value))
}

/// Converts bytes to hex string
fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// Converts hash to truncated hex string (first 8 chars)
fn hex_truncated(hash: &Hash) -> String {
    let full = hex_bytes(hash);
    if full.len() > 16 {
        format!("{}...{}", &full[..8], &full[full.len()-8..])
    } else {
        full
    }
}

/// Converts hash to full hex string
fn hex_full(hash: &Hash) -> String {
    hex_bytes(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut trie = MerklePatriciaTrie::new();
        trie.insert(b"key1", b"value1".to_vec());
        
        assert_eq!(trie.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(trie.get(b"key2"), None);
    }

    #[test]
    fn test_insert_update() {
        let mut trie = MerklePatriciaTrie::new();
        trie.insert(b"key", b"value1".to_vec());
        trie.insert(b"key", b"value2".to_vec());
        
        assert_eq!(trie.get(b"key"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_multiple_inserts() {
        let mut trie = MerklePatriciaTrie::new();
        trie.insert(b"do", b"verb".to_vec());
        trie.insert(b"dog", b"puppy".to_vec());
        trie.insert(b"doge", b"coin".to_vec());
        trie.insert(b"horse", b"stallion".to_vec());
        
        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
        assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
        assert_eq!(trie.get(b"doge"), Some(b"coin".to_vec()));
        assert_eq!(trie.get(b"horse"), Some(b"stallion".to_vec()));
    }

    #[test]
    fn test_delete() {
        let mut trie = MerklePatriciaTrie::new();
        trie.insert(b"key1", b"value1".to_vec());
        trie.insert(b"key2", b"value2".to_vec());
        
        assert_eq!(trie.get(b"key1"), Some(b"value1".to_vec()));
        
        trie.delete(b"key1");
        assert_eq!(trie.get(b"key1"), None);
        assert_eq!(trie.get(b"key2"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_root_hash_changes() {
        let mut trie = MerklePatriciaTrie::new();
        let initial_root = trie.root_hash();
        
        trie.insert(b"key", b"value".to_vec());
        let root_after_insert = trie.root_hash();
        
        assert_ne!(initial_root, root_after_insert);
        
        trie.delete(b"key");
        let root_after_delete = trie.root_hash();
        
        assert_eq!(initial_root, root_after_delete);
    }

    #[test]
    fn test_empty_trie() {
        let trie = MerklePatriciaTrie::new();
        assert_eq!(trie.get(b"anything"), None);
    }
}

