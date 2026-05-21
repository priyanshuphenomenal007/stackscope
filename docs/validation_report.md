# StackScope Validation Report

## Objective

Validate deterministic stack-depth estimation against:

- synthetic recursive firmware
- bounded acyclic firmware
- real ARM RTOS firmware
- compiler-generated stack metadata

---

# Validation Environment

| Component | Value |
|---|---|
| Host OS | Ubuntu 24.04 |
| Architecture | ARM Cortex-M |
| Compiler | arm-none-eabi-gcc 13.2.1 |
| RTOS | Zephyr |
| Analysis Engine | StackScope |
| Output Formats | JSON, SARIF |

---

# Validation Matrix

| Validation Case | Firmware Type | Expected Result | Actual Result | Status |
|---|---|---|---|---|
| `cortex_m_target` | Recursive | Unbounded recursion detected | Unbounded recursion detected | PASS |
| `cortex_m_target_acyclic` | Bounded firmware | Finite stack depth | 148 bytes | PASS |
| `zephyr_hello_world` | Real RTOS firmware | Finite stack depth | 152 bytes | PASS |

---

# Zephyr Ground Truth Comparison

## Compiler Stack Estimates

Compiler-generated `.su` files were extracted from Zephyr build artifacts.

Relevant call path:

```text
main
  -> printf
      -> vfprintf
```

---

## Comparison Results

| Source | Stack Estimate |
|---|---|
| GCC `.su` estimate | 160 bytes |
| StackScope estimate | 152 bytes |
| Absolute delta | 8 bytes |

---

## StackScope Critical Path

```text
main
  -> printf
      -> vfprintf
```

---

# Recursive Validation Result

Recursive firmware correctly triggered:

- unbounded stack growth detection
- deterministic policy violation
- CI-compatible non-zero exit code

Result:

```text
Exit code: 1
```

---

# Validation Harness

Validation execution is reproducible through:

```text
validation/scripts/run_validation.sh
```

Summary generation:

```text
validation/scripts/summarize_results.py
```

Generated machine-readable outputs:

```text
validation/results/
```

---

# Current Engine Capabilities

## Operational

- ARM ELF ingestion
- CFG recovery
- recursive cycle detection
- bounded stack propagation
- SARIF generation
- GitHub Actions integration
- deterministic CI exit semantics
- DWARF CFI frame extraction
- Zephyr firmware ingestion

---

## Known Limitations

Current limitations include:

- indirect call resolution incomplete
- function pointer modeling incomplete
- RTOS task-awareness absent
- ISR modeling absent
- DWARF parser tolerant but noisy
- stripped binary handling limited
- C++ firmware support unvalidated
- optimized firmware behavior not fully validated

---

# Validation Methodology

StackScope validation currently focuses on:

1. deterministic recursion detection
2. bounded stack propagation accuracy
3. comparison against compiler-generated metadata
4. reproducible CI-compatible execution semantics

The current methodology does not yet validate:

- full RTOS scheduling behavior
- interrupt preemption stacking
- dynamically resolved indirect calls
- highly optimized production firmware paths

---

# Operational Interpretation

The current validation results demonstrate that StackScope can:

- detect recursive stack hazards deterministically
- estimate bounded stack depth on real firmware
- generate machine-readable CI artifacts
- operate reproducibly across validation runs

The Zephyr comparison establishes initial alignment between StackScope propagation estimates and compiler-generated stack metadata.

---

# Conclusion

StackScope successfully analyzed:

- synthetic ARM firmware
- recursive firmware
- real Zephyr RTOS firmware

and produced stack estimates within:

```text
8 bytes
```

of compiler-generated stack metadata for the validated Zephyr firmware path.

This establishes the initial technical feasibility of deterministic firmware stack-depth analysis using StackScope.

---
