# Proof of Claw

**Provable Private Agent Framework**

ETHGlobal Cannes 2026 — Hackathon Submission

## Overview

Proof of Claw is a framework for running autonomous AI agents whose behavior is cryptographically provable, whose communication is end-to-end encrypted, and whose high-value actions require human approval via hardware signing.

### Key Features

- 🔒 **Private Inference** — Decentralized LLM reasoning via 0G Compute with TEE attestation
- 📦 **Decentralized Storage** — Persistent memory and execution traces on 0G Storage
- 🔐 **Encrypted Messaging** — Inter-agent communication via DM3 with ENS identity
- ✅ **Provable Compliance** — RISC Zero zkVM proofs of policy adherence
- 🔑 **Hardware Approval** — Ledger device integration for high-value actions

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     PROOF OF CLAW AGENT                         │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  Agent Core (Rust)                         │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │  │
│  │  │ Agent Loop   │  │ Tool Registry │  │ Safety Layer    │  │  │
│  │  │ (reasoning)  │  │ (WASM sandbox)│  │ (policy engine) │  │  │
│  │  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │  │
│  └─────────┼─────────────────┼────────────────────┼───────────┘  │
│            │                 │                    │               │
│  ┌─────────▼─────────────────▼────────────────────▼───────────┐  │
│  │                    Integration Layer                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │  │
│  │  │ 0G Compute   │  │ 0G Storage   │  │ ENS + DM3        │  │  │
│  │  │ RISC Zero    │  │ Ledger DMK   │  │                  │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Repository Structure

```
proof-of-claw/
├── agent/                      # Rust agent runtime
│   ├── src/
│   │   ├── core/               # Agent loop, intent router, job scheduler
│   │   ├── tools/              # WASM sandbox, tool registry, capabilities
│   │   ├── safety/             # Policy engine, sanitizer, injection detector
│   │   └── integrations/       # 0G, ENS/DM3, Ledger integrations
│   └── Cargo.toml
│
├── zkvm/                       # RISC Zero zkVM programs
│   ├── guest/                  # Guest program (policy verification)
│   ├── host/                   # Host program (proof generation)
│   └── Cargo.toml
│
├── contracts/                  # Solidity smart contracts
│   ├── src/
│   │   └── ProofOfClawVerifier.sol
│   ├── clear-signing/
│   │   └── proofofclaw.json    # ERC-7730 metadata
│   └── script/Deploy.s.sol
│
└── spec.md                     # Full technical specification
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Foundry (forge, cast)
- RISC Zero toolchain
- Node.js 18+ (for web UI, optional)

### 1. Build the Agent Runtime

```bash
cd agent
cargo build --release
```

### 2. Build RISC Zero Programs

```bash
cd zkvm
cargo build --release
```

### 3. Deploy Smart Contracts

```bash
cd contracts
forge build
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
```

### 4. Run the Agent

Create a `.env` file:

```env
AGENT_ID=alice-agent
ENS_NAME=alice.proofclaw.eth
PRIVATE_KEY=0x...
RPC_URL=https://eth-sepolia.g.alchemy.com/v2/...
ZERO_G_INDEXER_RPC=https://indexer-storage-testnet.0g.ai
ZERO_G_COMPUTE_ENDPOINT=https://broker-testnet.0g.ai
DM3_DELIVERY_SERVICE_URL=http://localhost:3001
ALLOWED_TOOLS=swap_tokens,transfer,query
MAX_VALUE_AUTONOMOUS_WEI=1000000000000000000
```

Run the agent:

```bash
cd agent
cargo run
```

## Compilation Status

✅ **Rust Agent Runtime** — Compiles successfully  
✅ **Solidity Contracts** — Compiles successfully  
⏳ **RISC Zero zkVM** — Ready for compilation (requires RISC Zero toolchain)

## Target Bounties

| Sponsor | Track | Prize | Integration |
|---------|-------|-------|-------------|
| **0G** | Best OpenClaw Agent on 0G | $6,000 | 0G Compute (inference), 0G Storage (memory + traces) |
| **ENS** | Best ENS Integration for AI Agents | $5,000 | ENS subnames for agent identity, DM3 for encrypted messaging |
| **Ledger** | AI Agents x Ledger | $6,000 | Ledger DMK/DSK for human approval, Clear Signing (ERC-7730) |

## Tech Stack

- **Agent Runtime**: Rust, Tokio, Wasmtime
- **Inference**: 0G Compute SDK with TEE attestation
- **Storage**: 0G Storage SDK
- **Identity**: ENS (ethers.js)
- **Messaging**: DM3 protocol
- **ZK Proofs**: RISC Zero zkVM + Boundless
- **Hardware Signing**: Ledger DMK/DSK
- **Smart Contracts**: Solidity (Foundry)

## Security Model

| Threat | Mitigation |
|--------|-----------|
| Agent acts outside policy | RISC Zero proof fails; action blocked on-chain |
| Inference tampering | 0G TEE attestation; signature in proof |
| Message interception | DM3 end-to-end encryption |
| Identity spoofing | ENS ownership tied to Ledger EOA |
| High-value action without consent | Ledger physical approval required |
| Prompt injection | Safety layer runs in proven execution trace |

## License

MIT

## Contact

Built for ETHGlobal Cannes 2026
=======
# project
>>>>>>> 108c7986d80c1579a2a38903992f08a6d48a2350
