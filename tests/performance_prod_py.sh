#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test file and script.

# Utility functions
# shellcheck disable=SC1091
source ./tests/utils.sh

function measure_performance_dev {
    python -V

    # Make virtual environment
    python -m venv .venv
    # Activate it
    source .venv/bin/activate
    # Show installed packages
    python -m pip freeze

    # Install most recent published version
    python -m pip install moss-decoder --upgrade --no-cache-dir --force-reinstall

    # Show installed packages
    python -m pip freeze

    cargo install hyperfine --locked

    hyperfine \
        "${BENCH_CMD}"\
        --warmup 3\
        --style full\
        --time-unit millisecond\
        --shell=bash\
        --export-markdown prod-bench.md
}

if [[ $# -eq 0 ]] ; then
    measure_performance_dev
fi
