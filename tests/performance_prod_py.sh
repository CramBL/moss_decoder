#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test file and script.

readonly SCRIPT_PATH="tests/integration.py"
readonly CMD="python ${SCRIPT_PATH}"

python -V

# Make virtual environment
python -m venv .venv
# Activate it
source .venv/bin/activate
# Show installed packages
python -m pip freeze

# Install most recent published version
python -m pip install moss-decoder --upgrade

# Show installed packages
python -m pip freeze

cargo install hyperfine --locked

hyperfine \
    "${CMD}"\
    --warmup 3\
    --style full\
    --time-unit millisecond\
    --shell=bash\
    --export-markdown prod-bench.md
