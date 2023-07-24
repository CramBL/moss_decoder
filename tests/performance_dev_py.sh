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

# install cargo and maturin
cargo install hyperfine --locked

#python -m pip install maturin

# Build and install the local package
maturin develop

# Check Moss version
python -m pip freeze

# Run benchmark
hyperfine \
    "${CMD}"\
    --warmup 3\
    --style full\
    --time-unit millisecond\
    --shell=bash\
    --export-markdown dev-bench.md

