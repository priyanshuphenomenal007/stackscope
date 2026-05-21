#!/bin/bash

set -e

ELF_PATH=$1
ENTRY_SYMBOL=$2
BUDGET=$3
FORMAT=$4

REPORT_FILE="stackscope-result.${FORMAT}"

echo "Running StackScope analysis..."
echo "ELF: ${ELF_PATH}"

set +e

stackscope analyze \
  --elf "${ELF_PATH}" \
  --entry "${ENTRY_SYMBOL}" \
  --budget "${BUDGET}" \
  --format "${FORMAT}" \
  > "${REPORT_FILE}"

EXIT_CODE=$?

set -e

# Only export GitHub outputs when running inside GitHub Actions
if [ -n "$GITHUB_OUTPUT" ]; then
  echo "report_file=${REPORT_FILE}" >> "$GITHUB_OUTPUT"
fi

exit $EXIT_CODE