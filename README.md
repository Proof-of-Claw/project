# Proof of Claw

**Provable Private Agent Framework**

ETHGlobal Cannes 2026 — Hackathon Submission

---

## Overview

Proof of Claw is a framework for running autonomous AI agents whose behavior is cryptographically provable, whose communication is end-to-end encrypted, and whose high-value actions require human approval via hardware signing.

The core agent runtime is adapted from [IronClaw](https://github.com/nearai/ironclaw), a Rust-based OpenClaw reimplementation with WASM-sandboxed tool execution, capability-based permissions, and defense-in-depth security.

### Key Features

- **Private Inference** — Decentralized LLM reasoning via 0G Compute (Sealed Inference TEE)
- **Decentralized Storage** — Persistent memory and execution traces on 0G Storage
- **Encrypted Messaging** — Inter-agent communication via DM3 with ENS identity resolution
- **Provable Compliance** — RISC Zero zkVM proofs of policy adherence, verified on-chain
- **Hardware Approval** — Ledger DMK/DSK integration with ERC-7730 Clear Signing for high-value actions
- **WASM Sandbox** — Untrusted tools execute in isolated Wasmtime containers with capability-based permissions
- **Swarm Protocol** — Multi-agent coordination and discovery via Swarm network

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     PROOF OF CLAW AGENT                         │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Agent Core — IronClaw Runtime                 │  │
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
│  │  │ (inference)  │  │ (traces)     │  │ (identity + msg) │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │  │
│  │  │ RISC Zero    │  │ Ledger DMK   │  │ Swarm Protocol   │  │  │
│  │  │ (ZK proofs)  │  │ (approval)   │  │ (coordination)   │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                              │                                    │
│                    ┌─────────▼──────────┐                         │
│                    │   On-Chain Layer    │                         │
│                    │  - ZK Verifier     │                         │
│                    │  - Policy Registry │                         │
│                    │  - Agent Vault     │                         │
│                    │  - ENS Resolver    │                         │
│                    └────────────────────┘                         │
└─────────────────────────────────────────────────────────────────┘
```

### Two-Tier Trust Model

| Tier | Condition | Signing | Verification |
|------|-----------|---------|-------------|
| **Autonomous** | Action value < threshold | Agent server wallet | RISC Zero proof on-chain |
| **Ledger-Gated** | Value >= threshold or escalation | Owner's Ledger device | RISC Zero + Ledger approval |

## Repository Structure

```
proof-of-claw/
├── agent/                      # Rust agent runtime
│   └── src/
│       ├── core/               # Agent loop, intent router, job scheduler
│       ├── tools/              # WASM sandbox, tool registry, capabilities
│       ├── safety/             # Policy engine, sanitizer, injection detector
│       └── integrations/       # 0G, ENS/DM3, Ledger integrations
│
├── zkvm/                       # RISC Zero zkVM programs
│   ├── guest/                  # Guest program (policy verification)
│   └── host/                   # Host program (proof generation)
│
├── contracts/                  # Solidity smart contracts
│   ├── src/
│   │   └── ProofOfClawVerifier.sol
│   ├── clear-signing/
│   │   └── proofofclaw.json    # ERC-7730 metadata
│   └── script/Deploy.s.sol
│
├── frontend/                   # Web UI
│   ├── index.html              # Landing page
│   ├── docs.html               # Documentation
│   ├── agents.html             # Agent management
│   ├── dashboard.html          # Monitoring dashboard
│   ├── deploy.html             # Agent deployment
│   ├── messages.html           # DM3 message viewer
│   └── proofs.html             # ZK proof explorer
│
├── spec.md                     # Full technical specification
├── ARCHITECTURE.md             # Detailed architecture docs
└── IRONCLAW_INTEGRATION.md     # IronClaw integration guide
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

### 4. Configure the Agent

Create a `.env` file in the `agent/` directory:

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

### 5. Run the Agent

```bash
cd agent
cargo run
```

## How It Works

1. Agent receives a task (user message or encrypted DM3 message from another agent)
2. Intent router classifies the action
3. 0G Compute performs private inference inside a TEE — prompts stay encrypted
4. Safety layer validates against the agent's declared policy
5. Tool execution happens in a WASM sandbox with capability-based permissions
6. Execution trace is stored on 0G Storage with content-addressable root hashes
7. RISC Zero proof of policy compliance is generated via Boundless
8. Agent submits proof + action to the on-chain verifier contract
9. If value exceeds threshold, Ledger approval is required — owner sees Clear Signing details on device

## Integrations

| Integration | Purpose | SDK |
|-------------|---------|-----|
| **0G Compute** | Private LLM inference via Sealed Inference TEE | `@0glabs/0g-serving-broker` |
| **0G Storage** | Decentralized execution trace storage | `@0glabs/0g-ts-sdk` |
| **ENS** | Agent identity via subnames (e.g. `alice-agent.proofclaw.eth`) | `ethers.js` |
| **DM3** | End-to-end encrypted inter-agent messaging | `@dm3-org/dm3-lib` |
| **RISC Zero** | ZK proofs of policy compliance | `risc0-zkvm` |
| **Boundless** | Decentralized proof generation network | Boundless SDK |
| **Ledger DMK** | Hardware-gated human approval | `@ledgerhq/device-management-kit` |
| **Ledger DSK** | Ethereum transaction signing | `@ledgerhq/device-signer-kit-ethereum` |
| **Swarm Protocol** | Multi-agent coordination and discovery | Swarm SDK |

## Compilation Status

- **Rust Agent Runtime** — Compiles successfully
- **Solidity Contracts** — Compiles successfully
- **RISC Zero zkVM** — Ready for compilation (requires RISC Zero toolchain)

## Security Model

| Threat | Mitigation |
|--------|-----------|
| Agent acts outside policy | RISC Zero proof fails; action blocked on-chain |
| Inference tampering | 0G Compute TEE attestation; signature in proof |
| Message interception | DM3 end-to-end encryption with keys from ENS profiles |
| Identity spoofing | ENS ownership tied to Ledger EOA |
| High-value action without consent | Physical Ledger approval with Clear Signing display |
| Prompt injection | Safety layer (injection detector + content sanitizer) in proven execution trace |

## Target Bounties

| Sponsor | Track | Prize | Integration |
|---------|-------|-------|-------------|
| **0G** | Best OpenClaw Agent on 0G | $6,000 | 0G Compute (inference), 0G Storage (memory + traces) |
| **ENS** | Best ENS Integration for AI Agents | $5,000 | ENS subnames for agent identity, DM3 for encrypted messaging |
| **Ledger** | AI Agents x Ledger | $6,000 | Ledger DMK/DSK for human approval, Clear Signing (ERC-7730) |

## Tech Stack

- **Agent Runtime**: Rust, Tokio, Wasmtime
- **Inference**: 0G Compute SDK (Sealed Inference TEE)
- **Storage**: 0G Storage SDK
- **Identity**: ENS (ethers.js)
- **Messaging**: DM3 protocol
- **ZK Proofs**: RISC Zero zkVM + Boundless
- **Hardware Signing**: Ledger DMK/DSK
- **Smart Contracts**: Solidity (Foundry)
- **Multi-Agent**: Swarm Protocol
- **Frontend**: Vanilla HTML/CSS/JS

## Documentation

Full documentation is available at [frontend/docs.html](frontend/docs.html), covering:

- Architecture deep-dive and system design
- Integration guides for each protocol (0G, ENS, DM3, RISC Zero, Ledger)
- Smart contract reference (ProofOfClawVerifier, ERC-7730 Clear Signing)
- Security threat model and safety layer details
- Configuration reference
- Repository structure

See also:
- [spec.md](spec.md) — Full technical specification
- [ARCHITECTURE.md](ARCHITECTURE.md) — Detailed architecture documentation
- [IRONCLAW_INTEGRATION.md](IRONCLAW_INTEGRATION.md) — IronClaw runtime integration guide

## License

MIT

## Contact

Built for ETHGlobal Cannes 2026
