import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ELF = ROOT / "validation/firmware/zephyr/build/zephyr/zephyr.elf"

STACKSCOPE_CMD = [
    "cargo",
    "run",
    "--",
    "analyze",
    "--elf",
    str(ELF),
    "--entry",
    "main",
    "--budget",
    "4096",
    "--format",
    "json",
]

SU_DIR = ROOT / "validation/ground_truth/zephyr"

FUNCTION_STACK = {}

SU_PATTERN = re.compile(
    r".*:\d+:\d+:(.*?)\s+(\d+)\s+"
)

for su_file in SU_DIR.glob("*.su"):
    with open(su_file, "r") as f:
        for line in f:
            match = SU_PATTERN.match(line.strip())

            if not match:
                continue

            function = match.group(1).strip()
            stack = int(match.group(2))

            FUNCTION_STACK[function] = stack

result = subprocess.run(
    STACKSCOPE_CMD,
    capture_output=True,
    text=True,
)

if result.returncode not in [0, 1]:
    print("StackScope execution failed")
    print(result.stderr)
    exit(1)

stdout = result.stdout

json_start = stdout.find("{")

if json_start == -1:
    print("No JSON payload found")
    print(stdout)
    exit(1)

payload = json.loads(stdout[json_start:])

critical_path = payload["result"]["critical_path"]

print("\n=== StackScope Critical Path ===\n")

compiler_total = 0

for symbol in critical_path:
    stack = FUNCTION_STACK.get(symbol)

    if stack is None:
        print(f"{symbol:<30} compiler-data-missing")
        continue

    compiler_total += stack

    print(f"{symbol:<30} {stack:>5} bytes")

print("\n================================")

print(f"Compiler estimate : {compiler_total} bytes")
print(
    f"StackScope estimate: "
    f"{payload['result']['max_depth_bytes']} bytes"
)

delta = abs(
    compiler_total - payload["result"]["max_depth_bytes"]
)

print(f"Delta              : {delta} bytes")