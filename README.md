# Stellar Quantum Messaging

A decentralized, quantum-secured messaging application built on the **Stellar blockchain**. Messages are encrypted using a simulated **BB84 Quantum Key Distribution** protocol and stored on-chain via a **Soroban smart contract**.

---

## Overview

Traditional encryption relies on mathematical hardness assumptions that could be broken by future quantum computers. This project demonstrates a **post-quantum-safe** messaging flow:

1. **BB84 Key Exchange** — A simulated quantum key distribution protocol generates a shared secret between sender and recipient.
2. **AES Encryption** — The BB84-derived key encrypts messages using AES-256.
3. **Stellar On-Chain Storage** — Encrypted ciphertext and integrity hashes are recorded on the Stellar testnet via a Soroban smart contract, providing an immutable, tamper-proof message registry.
4. **Recipient Acknowledgement** — Recipients can acknowledge message receipt on-chain, creating a verifiable delivery proof.

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    Next.js Frontend                   │
│  ┌──────────┐  ┌───────────┐  ┌────────────────────┐ │
│  │  BB84    │  │  AES      │  │  Stellar SDK       │ │
│  │  Key Gen │→ │  Encrypt  │→ │  Transaction Mgmt  │ │
│  └──────────┘  └───────────┘  └────────┬───────────┘ │
└────────────────────────────────────────┼─────────────┘
                                         │
                    ┌────────────────────▼──────────────┐
                    │    Stellar Testnet (Soroban)       │
                    │  ┌──────────────────────────────┐  │
                    │  │  Quantum Message Registry    │  │
                    │  │  • send_message()            │  │
                    │  │  • acknowledge_message()     │  │
                    │  │  • get_message()             │  │
                    │  │  • get_user_stats()          │  │
                    │  └──────────────────────────────┘  │
                    └───────────────────────────────────┘
```

---

## Project Structure

```
blockchain6/
├── app/
│   ├── lib/
│   │   ├── bb84.ts          # BB84 quantum key distribution simulation
│   │   ├── crypto.ts        # AES encryption / decryption utilities
│   │   └── stellar.ts       # Stellar SDK integration (keypair, transactions)
│   ├── globals.css           # Global styles
│   ├── layout.tsx            # Root layout component
│   └── page.tsx              # Main application UI
├── smart-contract/
│   ├── Cargo.toml            # Rust/Soroban project configuration
│   └── src/
│       └── lib.rs            # Soroban smart contract (Quantum Message Registry)
├── package.json
├── tsconfig.json
└── README.md
```

---

## Tech Stack

| Layer            | Technology                                                       |
| ---------------- | ---------------------------------------------------------------- |
| **Frontend**     | Next.js 16, React 19, TypeScript, Tailwind CSS 4                 |
| **Blockchain**   | Stellar Network (Testnet), Soroban Smart Contracts               |
| **SDK**          | `@stellar/stellar-sdk` v15                                       |
| **Encryption**   | AES-256 via `crypto-js`, BB84 QKD simulation                     |
| **Smart Contract** | Rust, `soroban-sdk` v22                                        |

---

## Smart Contract — Quantum Message Registry

The Soroban contract (`smart-contract/src/lib.rs`) provides on-chain message management:

### Functions

| Function               | Description                                           |
| ---------------------- | ----------------------------------------------------- |
| `initialize(admin)`    | One-time setup; sets the contract administrator       |
| `send_message(...)`    | Stores an encrypted message with sender, recipient, ciphertext, and content hash |
| `acknowledge_message(...)` | Recipient confirms receipt and successful decryption |
| `get_message(id)`      | Retrieve a message by its unique ID                   |
| `get_sent_messages(addr)` | List all message IDs sent by an address            |
| `get_received_messages(addr)` | List all message IDs received by an address    |
| `get_user_stats(addr)` | Retrieve per-user messaging statistics                |
| `get_message_count()`  | Total number of messages stored on-chain              |

### Data Model

```rust
struct EncryptedMessage {
    id: u64,
    sender: Address,
    recipient: Address,
    ciphertext: String,       // AES ciphertext (off-chain encryption)
    content_hash: BytesN<32>, // SHA-256 integrity hash
    timestamp: u64,           // Ledger sequence when recorded
    acknowledged: bool,       // Delivery confirmation
}
```

---

## BB84 Quantum Key Distribution

The BB84 module (`app/lib/bb84.ts`) simulates the quantum key exchange protocol:

1. **Alice** generates random bits and encodes them in random bases (rectilinear / diagonal).
2. **Bob** measures with independently chosen random bases.
3. **Basis Sifting** — They publicly compare bases and discard mismatches.
4. **Key Generation** — Matching measurements form the shared 256-bit secret key.

> **Note:** This is a classical simulation of BB84. A production system would require real quantum hardware for true quantum-secure key distribution.

---

## Getting Started

### Prerequisites

- **Node.js** ≥ 18
- **Rust** ≥ 1.74 (for smart contract compilation)
- **Soroban CLI** — `cargo install soroban-cli`
- **Stellar Testnet** account (auto-funded via Friendbot)

### Install & Run the Frontend

```bash
# Install dependencies
npm install

# Start the development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to access the application.

### Build & Deploy the Smart Contract

```bash
# Navigate to the contract directory
cd smart-contract

# Build the Wasm binary
soroban contract build

# Deploy to Stellar testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/quantum_message_registry.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>

# Initialize the contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <YOUR_SECRET_KEY> \
  -- initialize \
  --admin <YOUR_PUBLIC_KEY>
```

---

## Usage Flow

1. **Create Account** — Generate a Stellar keypair and fund it on testnet via Friendbot.
2. **Generate Quantum Key** — Run the BB84 simulation to produce a shared secret key.
3. **Share Key** — Securely share the generated key with the intended recipient (out-of-band).
4. **Send Message** — Compose a message, encrypt it with the quantum-derived key, and submit it to the Stellar network.
5. **Receive & Decrypt** — The recipient loads transactions, enters the shared key, and decrypts messages.

---

## Environment Variables

Create a `.env.local` file in the project root (optional):

```env
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_CONTRACT_ID=<deployed_contract_id>
```

---

## Testing

### Smart Contract Tests

```bash
cd smart-contract
cargo test
```

The contract includes tests for:
- Contract initialization
- Sending and retrieving messages
- Recipient acknowledgement
- User statistics tracking
- Sent/received message list tracking

### Frontend

```bash
npm run lint
npm run build
```

---

## License

MIT
