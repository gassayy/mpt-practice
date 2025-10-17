# Verifying Events in Ethereum Blocks

## The Problem

You want to verify that a specific event (e.g., a token transfer) was emitted in a specific block, but:
- You don't have the full blockchain (you're a light client)
- You don't want to trust a third party blindly
- You only have: parent hash, block hash, and the event details

**Question:** How can you cryptographically prove the event is real?

---

## Background: What Are Events in Ethereum?

### Events in Smart Contracts

```solidity
contract ERC20 {
    event Transfer(address indexed from, address indexed to, uint256 value);
    
    function transfer(address to, uint256 amount) public {
        balances[msg.sender] -= amount;
        balances[to] += amount;
        
        emit Transfer(msg.sender, to, amount);  // ← Creates a log entry
    }
}
```

**Events become logs** stored in the **receipt trie**.

---

## How Events Are Stored in Blocks

### Block Structure

```rust
struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
    // ... other data
}

struct BlockHeader {
    parent_hash: Hash,           // Previous block
    state_root: Hash,            // State trie root
    transactions_root: Hash,     // Transaction trie root
    receipts_root: Hash,         // Receipt trie root ← KEY!
    logs_bloom: [u8; 256],       // Bloom filter for logs
    // ... other fields
}
```

### Transaction Receipt Structure

```rust
struct TransactionReceipt {
    status: bool,                // Success/failure
    cumulative_gas_used: u64,    // Total gas in block so far
    logs_bloom: [u8; 256],       // Bloom filter for this tx's logs
    logs: Vec<Log>,              // ← Your events are here!
}

struct Log {
    address: Address,            // Contract that emitted it
    topics: Vec<Hash>,           // Indexed event parameters
    data: Vec<u8>,               // Non-indexed parameters
}
```

### Receipt Trie

All transaction receipts are stored in a Merkle Patricia Trie:

```
Receipt Trie (per block):
├─ Key: RLP(transaction_index)    // 0, 1, 2, ...
└─ Value: RLP(TransactionReceipt)
```

**The receipt trie root is in the block header!**

---

## Verification Process: Step by Step

### What You Have

```rust
// Known/trusted data:
let block_hash: Hash = /* Hash of the block header */;
let parent_hash: Hash = /* From block header */;

// Event you want to verify:
let event = Log {
    address: token_contract,
    topics: [
        keccak256("Transfer(address,address,uint256)"),
        keccak256(alice_address),  // from
        keccak256(bob_address),    // to
    ],
    data: encode_u256(1000),  // amount
};

// Transaction position:
let tx_index: u64 = 5;  // Event was in 6th transaction
```

### What You Need from Full Node

```rust
struct EventProof {
    // 1. Block header (you can verify this against block_hash)
    block_header: BlockHeader,
    
    // 2. Merkle proof for the receipt
    receipt_proof: MerkleProof {
        key: rlp_encode(tx_index),
        value: rlp_encode(receipt),
        nodes: Vec<Node>,  // Path through receipt trie
        root_hash: block_header.receipts_root,
    },
    
    // 3. The actual receipt containing your event
    receipt: TransactionReceipt,
}
```

### Verification Steps

```rust
fn verify_event_in_block(
    event: &Log,
    tx_index: u64,
    proof: EventProof,
    trusted_block_hash: Hash,
) -> bool {
    // Step 1: Verify block header integrity
    let computed_block_hash = keccak256(rlp_encode(&proof.block_header));
    if computed_block_hash != trusted_block_hash {
        return false;  // Block header doesn't match!
    }
    
    // Step 2: Verify receipt is in the receipt trie
    if !verify_merkle_proof(&proof.receipt_proof) {
        return false;  // Receipt not in trie!
    }
    
    // Step 3: Verify receipt matches the proof value
    let receipt_bytes = rlp_encode(&proof.receipt);
    if receipt_bytes != proof.receipt_proof.value {
        return false;  // Receipt data mismatch!
    }
    
    // Step 4: Verify event is in the receipt logs
    if !proof.receipt.logs.contains(event) {
        return false;  // Event not in this transaction!
    }
    
    // ✅ All checks passed!
    true
}
```

---

## Detailed Example: Token Transfer Event

### Scenario

```
Block #12345678
└─ Transaction 5: Alice transfers 1000 tokens to Bob
   └─ Event: Transfer(alice, bob, 1000)
```

### Step-by-Step Verification

#### 1. Get Trusted Block Hash

```rust
// You have the block hash from a trusted source
// (e.g., from Ethereum mainnet via multiple RPC providers)
let trusted_block_hash: Hash = 
    0xabcd1234...;  // Block #12345678
```

#### 2. Request Proof from Full Node

```javascript
// Using Ethereum JSON-RPC
const proof = await eth.getTransactionReceipt(tx_hash);
const merkle_proof = await eth.getProof(
    receipt_trie_address,
    [tx_index],
    block_number
);
```

#### 3. Verify Block Header

```rust
// Received block header
let block_header = proof.block_header;

// Compute hash
let computed_hash = keccak256(rlp_encode(&block_header));

assert_eq!(computed_hash, trusted_block_hash);
// ✓ Block header is authentic
```

#### 4. Verify Receipt Merkle Proof

```rust
// The receipt trie root is in the block header
let receipts_root = block_header.receipts_root;

// Key is the transaction index
let key = rlp_encode(5);  // Transaction #5

// Value is the receipt
let value = rlp_encode(&receipt);

// Verify the Merkle proof
verify_proof(key, value, proof.nodes, receipts_root);
// ✓ Receipt is in the trie
```

#### 5. Check Event in Receipt

```rust
// Look through the logs in the receipt
for log in receipt.logs {
    if log.address == token_contract &&
       log.topics[0] == keccak256("Transfer(address,address,uint256)") &&
       log.topics[1] == keccak256(alice) &&
       log.topics[2] == keccak256(bob) &&
       decode_u256(log.data) == 1000 {
        
        // ✓ Event found and verified!
        return true;
    }
}
```

---

## Optimization: Bloom Filters

Ethereum includes **bloom filters** to quickly check if an event *might* be in a block.

### Block Header Bloom Filter

```rust
struct BlockHeader {
    // ...
    logs_bloom: [u8; 256],  // 2048 bits
    // ...
}
```

**Quick Check:**
```rust
// Before requesting full proof, check bloom filter
if !block_header.logs_bloom.contains(&event_signature) {
    // Event is definitely NOT in this block
    return false;  // Skip expensive proof verification
}

// Event might be in block, need to verify with proof
```

**Benefits:**
- False positives: ~1% (might say yes when it's no)
- False negatives: 0% (never says no when it's yes)
- Quick rejection without downloading proofs

---

## Complete Verification Flow

### Light Client Workflow

```rust
// 1. Light client has trusted block hashes
let block_hash = get_trusted_block_hash(12345678);

// 2. Check bloom filter first (optional, but efficient)
let header = get_block_header(12345678);
if !header.logs_bloom.might_contain(&event_filter) {
    return false;  // Definitely not in this block
}

// 3. Request proof from full node
let proof = full_node.get_event_proof(
    block_number: 12345678,
    tx_index: 5,
    event_signature: "Transfer(address,address,uint256)"
);

// 4. Verify block header
assert_eq!(keccak256(rlp_encode(&proof.header)), block_hash);

// 5. Verify receipt Merkle proof
assert!(verify_merkle_proof(&proof.receipt_proof));

// 6. Check event is in receipt
assert!(proof.receipt.logs.contains(&expected_event));

// ✅ Event verified without downloading full blockchain!
```

---

## Trust Model

### What You Trust

1. **Block hash** - Obtained from:
   - Multiple RPC providers (consensus)
   - Trusted checkpoint
   - Your own light client sync

2. **Cryptographic hash functions** (Keccak-256)

3. **Block header structure** (Ethereum protocol rules)

### What You DON'T Trust

❌ Single RPC provider's raw data  
❌ Transaction execution claims  
❌ Event data without proof  
❌ Any state without merkle proof  

**You verify everything cryptographically!**

---

## Proof Size Analysis

### For a Single Event

```
Block Header:        ~500 bytes
Receipt Trie Proof:  ~5 nodes × 200 bytes = ~1 KB
Receipt Data:        ~200 bytes
Total:              ~1.7 KB
```

**Compare to:**
- Full block: ~100 KB
- Full state: ~1 TB

**Reduction:** 60,000× smaller than full block!

---

## Real-World Example: Cross-Chain Bridge

### Problem

A bridge needs to verify a token lock event on Ethereum before minting on another chain.

```solidity
// Ethereum mainnet
contract Bridge {
    event TokensLocked(address user, uint256 amount);
}

// Polygon/Arbitrum
contract BridgeMinter {
    // Need to verify TokensLocked event before minting
}
```

### Solution with Merkle Proofs

```rust
// Bridge validator on destination chain:

fn mint_tokens(
    user: Address,
    amount: u256,
    proof: EventProof,
) -> Result<()> {
    // 1. Verify block hash against Ethereum mainnet
    let eth_block_hash = ethereum_light_client.get_block_hash(proof.block_number);
    
    // 2. Verify event proof
    let event = Log {
        address: BRIDGE_CONTRACT,
        topics: [keccak256("TokensLocked(address,uint256)"), user],
        data: encode_u256(amount),
    };
    
    verify_event_in_block(&event, proof)?;
    
    // 3. Mint tokens (now proven!)
    mint(user, amount);
    
    Ok(())
}
```

**Security:** Cryptographically proven, no trusted relayer needed!

---

## Implementation with Your MPT

To support this, you'd add to your trie implementation:

### 1. Receipt Trie Builder

```rust
fn build_receipt_trie(receipts: Vec<TransactionReceipt>) -> MerklePatriciaTrie {
    let mut trie = MerklePatriciaTrie::new();
    
    for (index, receipt) in receipts.iter().enumerate() {
        let key = rlp_encode(index);
        let value = rlp_encode(receipt);
        trie.insert(&key, value);
    }
    
    trie
}
```

### 2. Event Proof Generation

```rust
fn generate_event_proof(
    block: &Block,
    tx_index: usize,
    event_filter: &EventFilter,
) -> Option<EventProof> {
    // Build receipt trie
    let receipt_trie = build_receipt_trie(&block.receipts);
    
    // Generate Merkle proof for this receipt
    let proof = receipt_trie.generate_proof(&rlp_encode(tx_index))?;
    
    // Verify event is in receipt
    let receipt = &block.receipts[tx_index];
    let event = receipt.logs.iter().find(|log| {
        event_filter.matches(log)
    })?;
    
    Some(EventProof {
        block_header: block.header,
        receipt_proof: proof,
        receipt: receipt.clone(),
        event: event.clone(),
    })
}
```

### 3. Event Verification

```rust
fn verify_event_proof(proof: &EventProof, trusted_block_hash: Hash) -> bool {
    // 1. Verify block header
    if keccak256(rlp_encode(&proof.block_header)) != trusted_block_hash {
        return false;
    }
    
    // 2. Verify receipt proof against receipts_root
    if !verify_proof(&proof.receipt_proof) {
        return false;
    }
    
    // 3. Verify event in receipt
    proof.receipt.logs.contains(&proof.event)
}
```

---

## Summary

### To Verify an Event with Just Block Hash

**You need:**
1. ✅ Trusted block hash
2. ✅ Block header (verify against block hash)
3. ✅ Merkle proof (receipt → receipts_root)
4. ✅ Receipt containing event

**Verification process:**
```
Block Hash (trusted)
    ↓ verify
Block Header
    ↓ contains
Receipts Root
    ↓ verify with Merkle proof
Receipt for Transaction #5
    ↓ contains
Your Event ✓
```

**Security:** Cryptographically proven with ~1.7 KB of data!

This is exactly how:
- Light clients verify transactions
- Cross-chain bridges work
- Block explorers provide proofs
- Ethereum fraud proofs function

Your MPT implementation is the foundation for all of this! 🚀

