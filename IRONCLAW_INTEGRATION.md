# IronClaw Integration Guide

## Overview

Proof of Claw now integrates with IronClaw as the base runtime, adding provable execution, encrypted messaging, and hardware approval on top of IronClaw's battle-tested WASM sandbox and safety layer.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROOF OF CLAW AGENT                          │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              IronClaw Runtime (Base)                       │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │  │
│  │  │ Agent Loop   │  │ WASM Sandbox │  │ Safety Layer    │  │  │
│  │  │ Multi-channel│  │ Capabilities │  │ Injection Guard │  │  │
│  │  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │  │
│  └─────────┼─────────────────┼────────────────────┼───────────┘  │
│            │                 │                    │               │
│  ┌─────────▼─────────────────▼────────────────────▼───────────┐  │
│  │              Proof of Claw Adapter Layer                    │  │
│  │                                                             │  │
│  │  • Intercepts IronClaw execution traces                    │  │
│  │  • Converts to Proof of Claw format                        │  │
│  │  • Routes LLM calls to 0G Compute                           │  │
│  │  • Stores traces on 0G Storage                             │  │
│  │  • Generates RISC Zero proofs                              │  │
│  │  • Handles DM3 encrypted messaging                         │  │
│  │  • Routes high-value actions to Ledger                     │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Build Modes

### Standalone Mode (Default)
Runs without IronClaw dependency - uses our lightweight implementation:

```bash
cargo build
cargo run
```

### IronClaw Integration Mode
Integrates with full IronClaw runtime:

```bash
cargo build --features ironclaw-integration
cargo run --features ironclaw-integration
```

## Integration Points

### 1. Execution Trace Conversion

IronClaw's execution traces are converted to Proof of Claw format:

```rust
// IronClaw trace format
IronClawExecutionTrace {
    session_id,
    tool_calls: [ToolCall],
    llm_interactions: [LLMInteraction],
    policy_checks: [PolicyCheck],
}

// Converted to Proof of Claw format
ExecutionTrace {
    agent_id,
    session_id,
    inference_commitment,  // From 0G Compute attestation
    tool_invocations,      // Hashed tool calls
    policy_check_results,  // IronClaw safety checks
    output_commitment,     // For RISC Zero proof
}
```

### 2. LLM Provider Interception

All LLM calls are routed through 0G Compute:

```rust
impl ProofOfClawHooks {
    async fn on_llm_call(&self, prompt: &str) -> Result<String> {
        // Route to 0G Compute instead of OpenAI/Anthropic
        self.adapter.intercept_llm_call(prompt).await
    }
}
```

### 3. Tool Execution Tracking

Every tool execution in IronClaw's WASM sandbox is tracked:

```rust
impl ProofOfClawHooks {
    async fn on_tool_execution(
        &self,
        tool_name: &str,
        input: &Value,
        output: &Value,
    ) -> Result<()> {
        // Track for proof generation
        // Store capability hash
        // Check against policy
    }
}
```

### 4. Session Completion

When IronClaw completes a session, we generate proofs:

```rust
impl ProofOfClawHooks {
    async fn on_session_complete(
        &self,
        trace: IronClawExecutionTrace,
        agent_id: &str,
    ) -> Result<()> {
        // Convert trace
        let proof_trace = self.adapter.convert_trace(trace, agent_id);
        
        // Store on 0G Storage
        let trace_hash = self.adapter.store_trace(&proof_trace).await?;
        
        // Generate RISC Zero proof
        // Submit to on-chain verifier
    }
}
```

## What We Get from IronClaw

✅ **WASM Sandbox** - Isolated tool execution with capability-based permissions  
✅ **Safety Layer** - Prompt injection defense, content sanitization  
✅ **Multi-channel Support** - REPL, HTTP, webhooks, web gateway  
✅ **Job Scheduler** - Parallel job execution, Docker sandbox orchestration  
✅ **Tool Registry** - Built-in, MCP, and WASM tool management  
✅ **Persistent Memory** - PostgreSQL storage with hybrid search  

## What We Add to IronClaw

🔐 **Provable Execution** - RISC Zero zkVM proofs of policy compliance  
🔒 **Private Inference** - 0G Compute  
📦 **Decentralized Storage** - 0G Storage for execution traces  
💬 **Encrypted Messaging** - DM3 protocol for inter-agent communication  
🔑 **Hardware Approval** - Ledger device integration for high-value actions  
⛓️ **On-chain Verification** - Smart contract proof verification  

## Compatibility Matrix

| Component | IronClaw | Proof of Claw | Status |
|-----------|----------|---------------|--------|
| WASM Sandbox | ✅ Wasmtime | ✅ Wasmtime | Compatible |
| Tool Capabilities | ✅ CapabilitiesFile | ✅ CapabilitiesFile | Compatible |
| Safety Layer | ✅ Pattern detection | ✅ Pattern detection | Compatible |
| Execution Traces | ✅ JSON format | ✅ Extended format | Adapter layer |
| LLM Provider | ✅ OpenAI/Anthropic | ✅ 0G Compute | Hook intercept |
| Storage | ✅ PostgreSQL | ✅ 0G Storage | Parallel storage |
| Messaging | ✅ HTTP/WebSocket | ✅ DM3 encrypted | Additional channel |
| Verification | ❌ None | ✅ RISC Zero + on-chain | New feature |

## Usage Example

```rust
use proof_of_claw_agent::ProofOfClawAgent;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AgentConfig::from_env()?;
    let agent = ProofOfClawAgent::new(config).await?;
    
    #[cfg(feature = "ironclaw-integration")]
    agent.run_with_ironclaw().await?;
    
    #[cfg(not(feature = "ironclaw-integration"))]
    agent.run_standalone().await?;
    
    Ok(())
}
```

## Testing

Run tests for both modes:

```bash
# Standalone mode tests
cargo test

# IronClaw integration tests
cargo test --features ironclaw-integration
```

## Next Steps

1. **Full IronClaw Integration** - Initialize IronClaw's Agent with our hooks
2. **Proof Generation Pipeline** - Connect execution traces to RISC Zero prover
3. **On-chain Submission** - Submit proofs to ProofOfClawVerifier contract
4. **DM3 Channel** - Add DM3 as an IronClaw channel
5. **Ledger Approval Flow** - Integrate Ledger approval into IronClaw's job scheduler

## Benefits of This Approach

✅ **Battle-tested Security** - Leverage IronClaw's proven WASM sandbox  
✅ **Rich Feature Set** - Get multi-channel, job scheduling, MCP support  
✅ **Backward Compatible** - Can run standalone without IronClaw  
✅ **Modular** - Clean adapter layer, easy to maintain  
✅ **Provable** - Add cryptographic verification without changing IronClaw core  
