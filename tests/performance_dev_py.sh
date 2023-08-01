#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test file and script.

# Utility functions
# shellcheck disable=SC1091
source ./tests/utils.sh

function measure_performance_prod {
    python -V

    # Make virtual environment
    python -m venv .venv
    # Activate it
    source .venv/bin/activate
    # Show installed packages
    python -m pip freeze

    # install cargo and maturin
    cargo install hyperfine --locked
    python -m pip install maturin

    # Build and install the local package
    maturin build --release

    # Get filename of the produced binary
    wheel_bin=$(ls -t target/wheels/ | head -n 1)
    # Install it
    python -m pip install "target/wheels/${wheel_bin}" --upgrade --no-cache-dir --force-reinstall

    # Show installed packages
    python -m pip freeze


    # Run benchmark
    hyperfine \
        "${BENCH_CMD}"\
        --warmup 3\
        --style full\
        --time-unit millisecond\
        --shell=bash\
        --export-markdown dev-bench.md
}


# Only run the performance test if this script is invoked with no arguments (to allow importing the function without running it)
if [[ $# -eq 0 ]] ; then
    measure_performance_prod
fi
