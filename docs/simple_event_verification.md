# Simple Guide: Event Verification with Visual Flow

## Overview: The Three MPT Tries in Ethereum

Ethereum uses **3 separate Merkle Patricia Tries** to organize data:

```
┌─────────────────────────────────────────────────────────────┐
│                    ETHEREUM BLOCK                           │
├─────────────────────────────────────────────────────────────┤
│  Block Header (contains 3 Merkle roots)                     │
│  ┌──────────────────────────────────────────────┐          │
│  │ • state_root      ──────┐                    │          │
│  │ • transactions_root ─────┼───┐               │          │
│  │ • receipts_root     ─────┼───┼───┐           │          │
│  └──────────────────────────┼───┼───┼───────────┘          │
└────────────────────────────┼───┼───┼───────────────────────┘
                              │   │   │
                              │   │   │
          ┌───────────────────┘   │   └───────────────────┐
          │                       │                       │
          ▼                       ▼                       ▼
    ┌─────────┐           ┌─────────────┐         ┌─────────────┐
    │ STATE   │           │TRANSACTIONS │         │  RECEIPTS   │
    │  TRIE   │           │    TRIE     │         │    TRIE     │
    └─────────┘           └─────────────┘         └─────────────┘
    (Accounts)            (Tx details)            (Logs/Events)
```

## 1. State Trie: Accounts Storage

### What it stores: All Ethereum accounts

```
State Trie Structure:
├─ Key: keccak256(address)          [32 bytes]
└─ Value: RLP(Account)

Account Structure:
├─ nonce: u64                       (transaction count)
├─ balance: u256                    (ETH in wei)
├─ storage_root: Hash               (→ points to contract's storage trie)
└─ code_hash: Hash                  (contract bytecode hash)
```

### Visual Example:

```
Alice's Account:
Address: 0x1234...5678

State Trie:
┌────────────────────────────────────────────────────┐
│ Key: keccak256(0x1234...5678)                      │
│      = 0xabcd...ef                                 │
│                                                    │
│ Value: Account {                                   │
│   nonce: 5,              ← Alice sent 5 txs       │
│   balance: 1000 ETH,     ← Alice's ETH balance    │
│   storage_root: 0x0000,  ← Not a contract         │
│   code_hash: 0x0000      ← No code                │
│ }                                                  │
└────────────────────────────────────────────────────┘
```

### For a Smart Contract:

```
Token Contract:
Address: 0xTOKEN...ADDR

State Trie:
┌────────────────────────────────────────────────────┐
│ Key: keccak256(0xTOKEN...ADDR)                     │
│                                                    │
│ Value: Account {                                   │
│   nonce: 1,                                       │
│   balance: 0 ETH,                                 │
│   storage_root: 0x9876...  ← Points to storage!  │
│   code_hash: 0x5432...     ← Contract code hash  │
│ }                                                  │
└────────────────────────────────────────────────────┘
                  │
                  │ storage_root points to ↓
                  │
          ┌───────▼──────────────────────────┐
          │   Contract Storage Trie          │
          │                                  │
          │  Key: keccak256(slot_number)     │
          │  Value: stored_data              │
          │                                  │
          │  balances[alice] = 1000 tokens   │
          │  balances[bob] = 500 tokens      │
          └──────────────────────────────────┘
```

## 2. Transactions Trie: Transaction Data

### What it stores: All transactions in this block

```
Transaction Trie Structure:
├─ Key: RLP(index)                  [0, 1, 2, ...]
└─ Value: RLP(Transaction)

Transaction Structure:
├─ from: Address
├─ to: Address
├─ value: u256                      (ETH amount)
├─ gas_limit: u64
├─ gas_price: u256
├─ nonce: u64
└─ data: Vec<u8>                    (function call data)
```

### Visual Example:

```
Block with 3 transactions:

Transactions Trie:
┌─────────────────────────────────────────────────┐
│ Key: 0  →  Value: Transaction {                 │
│              from: Alice,                       │
│              to: Bob,                           │
│              value: 10 ETH                      │
│            }                                    │
├─────────────────────────────────────────────────┤
│ Key: 1  →  Value: Transaction {                 │
│              from: Carol,                       │
│              to: TokenContract,                 │
│              data: transfer(Bob, 1000)          │
│            }                                    │
├─────────────────────────────────────────────────┤
│ Key: 2  →  Value: Transaction {                 │
│              from: Dave,                        │
│              to: NFTContract,                   │
│              data: mint(1)                      │
│            }                                    │
└─────────────────────────────────────────────────┘
```

## 3. Receipts Trie: Execution Results & Events

### What it stores: Results of transaction execution

```
Receipts Trie Structure:
├─ Key: RLP(index)                  [Same index as transaction]
└─ Value: RLP(Receipt)

Receipt Structure:
├─ status: bool                     (success/fail)
├─ cumulative_gas_used: u64         (total gas used in block)
├─ logs_bloom: [u8; 256]           (quick filter)
└─ logs: Vec<Log>                   ← EVENTS ARE HERE!

Log (Event) Structure:
├─ address: Address                 (contract that emitted)
├─ topics: Vec<Hash>                (indexed parameters)
└─ data: Vec<u8>                    (non-indexed parameters)
```

### Visual Example:

```
Receipt for Transaction #1 (Carol → Token.transfer):

Receipts Trie:
┌─────────────────────────────────────────────────┐
│ Key: 1  (matches transaction index)             │
│                                                 │
│ Value: Receipt {                                │
│   status: true,          ← Success!            │
│   gas_used: 52000,                             │
│   logs: [                                      │
│     Log {                ← THE EVENT!          │
│       address: TokenContract,                  │
│       topics: [                                │
│         keccak256("Transfer(address,address,uint256)"),│
│         Carol,           ← from                │
│         Bob              ← to                  │
│       ],                                       │
│       data: encode(1000) ← amount              │
│     }                                          │
│   ]                                            │
│ }                                              │
└─────────────────────────────────────────────────┘
```

## Complete Flow: From Transaction to Event Verification

### Step 1: Transaction Execution

```
User Action:
┌──────────────────────────────────┐
│ Carol calls:                     │
│ Token.transfer(Bob, 1000)        │
└──────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────┐
│ Creates Transaction              │
│ • from: Carol                    │
│ • to: TokenContract              │
│ • data: transfer(Bob, 1000)      │
└──────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────┐
│ Miner/Validator includes in      │
│ Block #12345, position 1         │
└──────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────┐
│ EVM Executes Transaction         │
│ 1. Check Carol's balance         │
│ 2. Check Carol's token balance   │
│ 3. Update storage:               │
│    balances[Carol] -= 1000       │
│    balances[Bob] += 1000         │
│ 4. Emit Transfer event           │
└──────────────────────────────────┘
         │
         ├─────────────┬──────────────┐
         │             │              │
         ▼             ▼              ▼
  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │  STATE   │  │   TXS    │  │ RECEIPTS │
  │  TRIE    │  │   TRIE   │  │   TRIE   │
  │ updated  │  │ tx added │  │ log added│
  └──────────┘  └──────────┘  └──────────┘
         │             │              │
         └─────────────┴──────────────┘
                       │
                       ▼
         ┌──────────────────────────┐
         │   Block Header Created   │
         │ • state_root = hash₁     │
         │ • txs_root = hash₂       │
         │ • receipts_root = hash₃  │
         └──────────────────────────┘
```

### Step 2: How Everything Links Together

```
                    Block Header
                  ┌─────────────┐
                  │ Block #12345│
                  ├─────────────┤
                  │state_root   │───────┐
                  │txs_root     │───┐   │
                  │receipts_root│─┐ │   │
                  └─────────────┘ │ │   │
                                  │ │   │
    ┌─────────────────────────────┘ │   │
    │                 ┌──────────────┘   │
    │                 │     ┌────────────┘
    │                 │     │
    ▼                 ▼     ▼
┌─────────┐     ┌─────────┐     ┌─────────┐
│Receipts │     │   Txs   │     │  State  │
│  Trie   │     │  Trie   │     │  Trie   │
└─────────┘     └─────────┘     └─────────┘
    │                 │               │
    │ Index: 1        │ Index: 1      │ Carol's account
    ▼                 ▼               ▼
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Receipt │     │   Tx    │     │ Account │
│ with    │     │ details │     │  nonce++│
│ Transfer│     │         │     │ balance │
│  event  │     │         │     │  etc.   │
└─────────┘     └─────────┘     └─────────┘
                                      │
                         ┌────────────┘
                         │ storage_root
                         ▼
                  ┌──────────────┐
                  │Contract      │
                  │Storage Trie  │
                  │              │
                  │balances[Carol]│
                  │balances[Bob]  │
                  └──────────────┘
```

## Event Verification Flow (The Key Part!)

### Scenario: Light Client Wants to Verify Transfer Event

```
Light Client Has:
┌────────────────────────────┐
│ • Block hash (trusted)     │
│ • Event details            │
│   - from: Carol            │
│   - to: Bob                │
│   - amount: 1000           │
└────────────────────────────┘
```

### Verification Flow Chart:

```
START: Light Client wants to verify event
│
├─ Step 1: Get Block Header from Full Node
│   │
│   ├─ Receives: Block Header
│   │   ┌──────────────────────────┐
│   │   │ • parent_hash            │
│   │   │ • state_root             │
│   │   │ • txs_root               │
│   │   │ • receipts_root: Hash₃   │ ← IMPORTANT!
│   │   │ • logs_bloom             │
│   │   └──────────────────────────┘
│   │
│   ├─ Verify: keccak256(header) == trusted_block_hash
│   │   ✓ PASS → Continue
│   │   ✗ FAIL → REJECT (tampered header)
│   │
│
├─ Step 2: (Optional) Quick Check with Bloom Filter
│   │
│   ├─ Check: header.logs_bloom.contains("Transfer")
│   │   ✓ Maybe present → Continue
│   │   ✗ Definitely absent → STOP (not in block)
│   │
│
├─ Step 3: Request Merkle Proof from Full Node
│   │
│   ├─ Request: Proof for transaction #1's receipt
│   │
│   ├─ Receives: MerkleProof {
│   │   key: RLP(1),
│   │   value: RLP(Receipt { logs: [...] }),
│   │   nodes: [Node₁, Node₂, Node₃, ...], ← Path through trie
│   │   root_hash: Hash₃
│   │   }
│   │
│
├─ Step 4: Verify Merkle Proof
│   │
│   ├─ Process:
│   │   ┌────────────────────────────────────┐
│   │   │ a) Start with leaf (Receipt)       │
│   │   │    Hash(RLP(Receipt)) → Hash_A     │
│   │   │                                    │
│   │   │ b) Check Hash_A in parent Node₁    │
│   │   │    Hash(Node₁) → Hash_B            │
│   │   │                                    │
│   │   │ c) Check Hash_B in parent Node₂    │
│   │   │    Hash(Node₂) → Hash_C            │
│   │   │                                    │
│   │   │ d) Continue to root                │
│   │   │    Final Hash == Hash₃?            │
│   │   └────────────────────────────────────┘
│   │
│   ├─ Verify: computed_root == header.receipts_root (Hash₃)
│   │   ✓ PASS → Receipt is authentic
│   │   ✗ FAIL → REJECT (invalid proof)
│   │
│
├─ Step 5: Check Event in Receipt
│   │
│   ├─ Parse Receipt.logs[]
│   ├─ Find Log matching:
│   │   • address == TokenContract
│   │   • topics[0] == keccak256("Transfer(address,address,uint256)")
│   │   • topics[1] == Carol
│   │   • topics[2] == Bob
│   │   • decode(data) == 1000
│   │
│   ├─ Found?
│   │   ✓ YES → EVENT VERIFIED! ✓✓✓
│   │   ✗ NO → Event not in this tx
│   │
│
END: Event cryptographically proven
```

## Visual: What Gets Hashed and When

```
Transaction Execution:
┌──────────────────────────────────────────────────────────┐
│                                                          │
│  ┌─────────┐      ┌──────────┐      ┌─────────┐        │
│  │Contract │ emits│   Log    │ goes │ Receipt │        │
│  │  code   │─────→│ (Event)  │────→│with logs│        │
│  └─────────┘      └──────────┘  in  └─────────┘        │
│                                          │              │
│                                          │              │
│                      ┌───────────────────┘              │
│                      │ Store in Receipts Trie           │
│                      ▼                                  │
│              ┌────────────────┐                         │
│              │ Receipts Trie  │                         │
│              │ ┌────┬────┬───┐│                         │
│              │ │ 0  │ 1  │ 2 ││                         │
│              │ └────┴────┴───┘│                         │
│              └────────┬────────┘                         │
│                       │ RLP + Hash                       │
│                       ▼                                  │
│              ┌──────────────────┐                        │
│              │  receipts_root   │                        │
│              │  (32 bytes hash) │                        │
│              └────────┬─────────┘                        │
└──────────────────────┼──────────────────────────────────┘
                       │
                       │ Goes into Block Header
                       ▼
              ┌──────────────────┐
              │  Block Header    │
              │ ┌──────────────┐ │
              │ │receipts_root │ │
              │ │state_root    │ │
              │ │txs_root      │ │
              │ └──────────────┘ │
              └────────┬─────────┘
                       │ RLP + Hash
                       ▼
              ┌──────────────────┐
              │   Block Hash     │
              │  (32 bytes hash) │ ← This is what you trust!
              └──────────────────┘
```

## Size Comparison: Full Node vs Light Client

```
FULL NODE:
┌─────────────────────────────────────────────────────┐
│ Entire State Trie:        ~1 TB                     │
│ All Transactions:         ~500 GB                   │
│ All Receipts:            ~300 GB                    │
│ Total:                   ~1.8 TB                    │
└─────────────────────────────────────────────────────┘

LIGHT CLIENT (with Merkle Proof):
┌─────────────────────────────────────────────────────┐
│ Block Headers only:       ~80 GB (all blocks)       │
│ For ONE event proof:                                │
│   • Block header:         ~500 bytes                │
│   • Merkle proof nodes:   ~1 KB                     │
│   • Receipt data:         ~200 bytes                │
│   Total per event:        ~1.7 KB                   │
└─────────────────────────────────────────────────────┘

Reduction: 1,000,000× smaller for single event!
```

## Example: Real Transfer Verification

```
Scenario:
┌────────────────────────────────────────┐
│ Alice sent 100 USDT to Bob             │
│ Block: 18,500,000                      │
│ Transaction: #47 in that block         │
└────────────────────────────────────────┘

What Light Client Does:

1. Trust:
   block_hash = 0xabcd1234... (from multiple RPCs)

2. Get block header → Verify hash matches ✓

3. Get proof for tx #47's receipt from full node

4. Verify proof:
   Receipt for tx #47
       ↓ (Merkle proof nodes)
   receipts_root in header
       ↓ (header hash)
   Trusted block_hash ✓

5. Check receipt contains:
   Transfer(Alice, Bob, 100 USDT) ✓

6. Result: VERIFIED! Alice definitely sent 100 USDT to Bob
```

## Summary: Key Concepts

### Three Tries, Three Purposes

| Trie | Key | Value | Purpose |
|------|-----|-------|---------|
| **State** | address hash | Account | Who has how much ETH/code |
| **Transactions** | tx index | Transaction | What actions were requested |
| **Receipts** | tx index | Receipt+Logs | What actually happened (events!) |

### The Trust Chain

```
Trusted Block Hash (32 bytes)
    ↓
Block Header (verified by hash)
    ↓
Receipts Root (in header)
    ↓
Merkle Proof (~1 KB)
    ↓
Receipt with Event (verified!)
```

### Why This Works

1. **Cryptographic Hashing**: Can't fake without changing hash
2. **Merkle Trees**: Can prove inclusion with small proof
3. **Block Headers**: Link everything together
4. **Trust Anchor**: Only need to trust one 32-byte hash

---

This is how Ethereum enables trustless verification at scale! 🚀

