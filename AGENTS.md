# AGENTS.md

## Cursor Cloud specific instructions

### Repository structure

This is a single product ("Proof of Claw") with 4 components. See `ARCHITECTURE.md` and `README.md` for full details.

| Component | Path | Build | Test |
|-----------|------|-------|------|
| Agent Runtime (IronClaw + proof_of_claw) | `agent/` (git submodule) | `cd agent && cargo build` | `cd agent && cargo test` |
| Smart Contracts (Foundry) | `contracts/` | `forge build` | `forge test` (no tests currently) |
| zkVM Programs | `zkvm/` | `cd zkvm && cargo build --release` | N/A (needs RISC Zero toolchain) |
| Frontend (vanilla HTML/CSS/JS) | `frontend/` | No build step | Serve with `python3 -m http.server` |

### Git submodules

The `agent/` directory is a git submodule pointing to `https://github.com/Proof-of-Claw/ironclaw.git` (branch `claudeSlop`). Run `git submodule update --init agent` and `git submodule update --init contracts/lib/forge-std` before building. There is also a `zkvm/clawcode` entry in `.git/modules` without a `.gitmodules` mapping — this can be ignored.

### Rust toolchain

The workspace requires Rust edition 2024 (`rust-version = "1.92"`). Run `rustup default stable` to ensure the latest stable toolchain is active.

### Foundry

Foundry (`forge`) must be on PATH. After installation with `foundryup`, add `$HOME/.foundry/bin` to PATH.

### Running the agent

The agent binary is `ironclaw`. Key CLI flags for headless/CI usage:

- `cargo run -- run --no-onboard --no-db` — skip interactive onboarding wizard, skip database
- `cargo run -- run --no-onboard --no-db --message "..."` — single-message mode (requires an LLM backend configured)
- Before first run, set `onboard_completed` and `profile_onboarding_completed` to `true`:
  ```
  cargo run -- config set onboard_completed true
  cargo run -- config set profile_onboarding_completed true
  ```
- If a stale PID file prevents starting, remove `~/.ironclaw/ironclaw.pid`.
- The gateway listens on port 3000, webhook server on port 8080 by default.
- LLM inference requires configuring a provider (e.g., `cargo run -- config set llm_backend ollama`). Without a configured provider, the agent starts but cannot process chat messages.

### Running the frontend

```
cd frontend && python3 -m http.server 8888
```

Then open `http://localhost:8888/agents.html`. No build step or npm dependencies needed.

### Makefile targets

Standard targets in the root `Makefile`: `build-agent`, `build-contracts`, `test-agent`, `test-contracts`, `run-agent`, `check`, `clean`. See `Makefile` for details.
