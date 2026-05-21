# StackScope

Static stack analysis and stack regression detection for embedded firmware.

StackScope analyzes compiled ELF binaries to estimate worst-case stack depth (WCSD), detect recursive call cycles, and enforce stack budgets in CI pipelines.

The project is designed around deterministic outputs, machine-readable policies, and CI/CD integration.

---

# Current Status

StackScope is an early-stage prototype focused on:

- ELF ingestion
- call graph construction
- recursive cycle detection
- worst-case stack depth estimation
- stack regression diffing
- SARIF export for CI systems
- Dockerized execution

The current implementation has been validated locally using Docker and GitHub Actions-compatible SARIF output.

---

# Core Capabilities

## Analyze Firmware Binaries

Analyze a single ELF binary:

```bash
cargo run -- analyze \
  --elf fixtures/cortex_m_target/target.elf \
  --budget 100 \
  --format text
```

Example JSON output:

```json
{
  "schema_version": "1.0",
  "analysis_type": "analyze",
  "result": {
    "budget_exceeded": true,
    "critical_path": [
      "main",
      "recursive_function"
    ],
    "is_unbounded": true,
    "max_depth_bytes": 144
  }
}
```

---

## Stack Regression Detection

Compare two firmware binaries:

```bash
cargo run -- diff \
  --baseline old.elf \
  --candidate new.elf \
  --format json
```

The diff mode detects:

- stack growth regressions
- newly introduced recursive cycles
- changes in critical stack paths

---

## SARIF Export

StackScope can emit SARIF 2.1.0 for CI integration:

```bash
cargo run -- analyze \
  --elf firmware.elf \
  --budget 4096 \
  --format sarif
```

This allows integration with platforms that support SARIF ingestion, including GitHub code scanning workflows.

---

## Dockerized Execution

Build the analysis container:

```bash
docker build \
  -t stackscope-action \
  . \
  -f .github/actions/stackscope/Dockerfile
```

Run locally:

```bash
docker run --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  stackscope-action \
  fixtures/cortex_m_target/target.elf \
  main \
  100 \
  sarif
```

Generated SARIF report:

```bash
cat stackscope-result.sarif
```

---

# Operational Model

StackScope operates using static analysis over compiled firmware artifacts.

Primary analysis stages:

1. ELF parsing
2. execution graph extraction
3. call graph traversal
4. recursive cycle detection
5. stack accumulation analysis
6. policy evaluation
7. SARIF or structured report generation

The current implementation prioritizes:

- deterministic analysis
- CI/CD compatibility
- reproducible outputs
- low operational overhead
- offline execution

The project intentionally avoids:

- runtime instrumentation
- emulator-based execution tracing
- probabilistic estimation models
- cloud-based analysis services

---

# Exit Codes

StackScope uses deterministic exit codes for CI enforcement.

| Exit Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Policy violation |
| `2` | Invalid configuration |
| `3` | ELF parsing failure |
| `4` | Entry point not found |
| `5` | Unsupported architecture |
| `6` | Internal analysis failure |

---

# Project Structure

```text
src/
├── ege/    # ELF and execution graph extraction
├── mga/    # Memory geometry abstractions
├── sdt/    # Stack divergence and WCSD analysis
└── types/  # Shared graph and domain models
```

---

# Current Limitations

Current implementation limitations include:

- indirect function pointer resolution is incomplete
- RTOS task-switching semantics are not modeled
- interrupt nesting behavior is not modeled
- recursion is treated as unbounded stack growth
- DWARF source-level location mapping is limited
- stack estimation accuracy depends on call graph completeness

The project is currently best suited for:

- experimentation
- CI prototyping
- research workflows
- embedded tooling evaluation

---

# Intended Usage

StackScope is designed primarily for:

- firmware stack-budget enforcement
- embedded CI/CD gating
- regression detection
- static firmware analysis
- engineering validation workflows

It is not intended to replace:

- full compiler verification systems
- symbolic execution engines
- hardware-in-the-loop testing
- runtime profiling systems

---

# Roadmap

Near-term priorities:

- GitHub Action publication
- stable SARIF contracts
- DOT/graph export
- DWARF source mapping
- golden snapshot testing
- architecture-specific stack modeling

Longer-term areas under investigation:

- RTOS-aware stack partitioning
- ISR-aware stack accounting
- function pointer resolution
- policy packs and governance workflows

---

# Development

## Build

```bash
cargo build
```

---

## Format

```bash
cargo fmt
```

---

## Run Tests

```bash
cargo test
```

---

# Design Constraints

The project intentionally prioritizes:

- deterministic output
- static analysis transparency
- machine-readable reporting
- CI portability
- low operational complexity

over:

- runtime tracing infrastructure
- dynamic instrumentation
- distributed analysis systems
- autonomous remediation workflows

---

# License

TBD