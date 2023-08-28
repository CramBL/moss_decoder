#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test files and script.

# Utility functions
# shellcheck disable=SC1091
source ./tests/utils.sh

function py_integration_tests {
    python -V


    println_cyan "Make virtual environment"
    python -m venv .venv

    println_cyan "Activate virtual environment"
    source .venv/bin/activate

    println_cyan "Show installed packages"
    python -m pip freeze

    println_cyan "install hyperfine and maturin"
    cargo install hyperfine --locked
    python -m pip install maturin

    println_cyan "Build and install the local package"
    maturin build --release

    println_cyan "Get filename of the produced binary"
    wheel_bin=$(ls -t target/wheels/ | head -n 1)

    println_cyan "Install produced wheel binary"
    python -m pip install "target/wheels/${wheel_bin}" --upgrade --no-cache-dir --force-reinstall

    println_cyan "Show installed packages"
    python -m pip freeze


    println_cyan "Run Python integration tests"
    python tests/integration.py
}


# Only run the integration test if this script is invoked with no arguments (to allow importing the function without running it)
if [[ $# -eq 0 ]] ; then
    py_integration_tests
fi
