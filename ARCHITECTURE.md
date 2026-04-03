# Proof of Claw Architecture

## System Components

### 1. Agent Runtime (Rust)

The core agent is written in Rust and consists of:

#### Core Components
- **Agent Loop** (`agent/src/core/agent.rs`) - Main event loop handling messages and coordination
- **Intent Router** (`agent/src/core/intent_router.rs`) - Classifies incoming messages into actionable intents
- **Job Scheduler** (`agent/src/core/job_scheduler.rs`) - Manages asynchronous task execution
- **Configuration** (`agent/src/core/config.rs`) - Environment-based configuration management

#### Tools System
- **Tool Registry** (`agent/src/tools/registry.rs`) - Manages available tools and their metadata
- **WASM Sandbox** (`agent/src/tools/sandbox.rs`) - Isolated execution environment for untrusted tools
- **Capabilities** (`agent/src/tools/capabilities.rs`) - Permission and rate limit enforcement

#### Safety Layer
- **Policy Engine** (`agent/src/safety/policy_engine.rs`) - Enforces agent policy rules
- **Content Sanitizer** (`agent/src/safety/sanitizer.rs`) - Removes malicious content
- **Injection Detector** (`agent/src/safety/injection_detector.rs`) - Detects prompt injection attempts

#### Integrations
- **0G Compute** (`agent/src/integrations/zero_g.rs`) - Private inference
- **0G Storage** (`agent/src/integrations/zero_g.rs`) - Decentralized trace storage
- **ENS/DM3** (`agent/src/integrations/ens_dm3.rs`) - Identity and encrypted messaging
- **Ledger** (`agent/src/integrations/ledger.rs`) - Hardware approval gateway

### 2. RISC Zero zkVM

#### Guest Program (`zkvm/guest/src/main.rs`)
Runs inside the zkVM to verify:
- Tool invocations match allowed list
- All policy checks passed
- Action value vs autonomous threshold
- Outputs policy hash and approval requirement

#### Host Program (`zkvm/host/src/main.rs`)
Orchestrates proof generation:
- Builds execution environment
- Submits to Boundless proving network
- Returns verifiable receipt

### 3. Smart Contracts (Solidity)

#### ProofOfClawVerifier (`contracts/src/ProofOfClawVerifier.sol`)
- Agent registration with policy commitment
- RISC Zero proof verification
- Autonomous vs Ledger-gated execution routing
- Action execution with on-chain enforcement

#### Clear Signing Metadata (`contracts/clear-signing/proofofclaw.json`)
ERC-7730 metadata for human-readable Ledger display

## Data Flow

### Autonomous Action Flow

1. Agent receives message (user or DM3)
2. Intent router classifies action
3. 0G Compute performs private inference
4. Safety layer validates against policy
5. Tool execution in WASM sandbox
6. Execution trace stored on 0G Storage
7. RISC Zero proof generated via Boundless
8. Agent wallet submits proof + action to verifier contract
9. Contract verifies proof and executes action

### Ledger-Gated Action Flow

1-7. Same as autonomous flow
8. RISC Zero proof indicates approval required
9. Contract emits `ApprovalRequired` event
10. Web UI alerts owner
11. Owner reviews on Ledger device (Clear Signing)
12. Owner physically approves
13. Ledger signs `approveAction()` transaction
14. Contract executes action

### Inter-Agent Messaging Flow

1. Agent A resolves Agent B's ENS name
2. Retrieves B's DM3 profile (encryption key, delivery service)
3. Verifies B's policy via `proofclaw.imageId` text record
4. Encrypts message with B's public key
5. Sends to B's delivery service
6. B receives, decrypts, evaluates against policy
7. B responds via same encrypted channel
8. If both agree and value high, both route to Ledger
9. Upon dual approval, both execute with proofs

## Security Boundaries

### Trust Assumptions
- **Trusted**: Ledger device, RISC Zero verifier, 0G infrastructure
- **Untrusted**: Agent server, tool code, LLM responses, inter-agent messages

### Isolation Layers
1. **WASM Sandbox** - Tools run in isolated Wasmtime environment
2. **Inference Attestation** - Inference responses cryptographically signed
3. **ZK Proof** - Agent behavior proven without revealing private data
4. **Ledger Approval** - Physical confirmation for high-value actions

## Performance Characteristics

- **Agent Loop**: ~10ms per message (excluding inference)
- **0G Inference**: ~1-5s per LLM call (depending on model)
- **0G Storage**: ~100-500ms per trace upload
- **RISC Zero Proof**: ~30s-2min via Boundless (depending on complexity)
- **On-chain Verification**: ~50-100k gas per proof verification

## Scalability Considerations

- Agent can handle ~100 messages/sec (I/O bound)
- RISC Zero proving parallelizable via Boundless network
- 0G Storage scales horizontally
- DM3 delivery service can be federated
- Multiple agents can share same policy program (image ID)
