#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test file and script.

readonly SCRIPT_PATH="tests/integration.py"
readonly CMD="python ${SCRIPT_PATH} benchmark"

python -V

# Make virtual environment
python -m venv .venv
# Activate it
source .venv/bin/activate
# Show installed packages
python -m pip freeze

# install cargo and maturin
cargo install hyperfine --locked

# Build and install the local package
maturin build --release

# Get filename of the produced binary
wheel_bin=$(ls -t target/wheels/ | head -n 1)
# Install it
python -m pip install "target/wheels/${wheel_bin}" --upgrade --no-cache-dir --force-reinstall

# Show installed packages
python -m pip freeze


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
