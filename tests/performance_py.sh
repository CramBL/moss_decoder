#!/bin/bash

# Prerequisites: cargo, python >= 3.7, pip, test file and script.

readonly SCRIPT_PATH="tests/integration.py"
readonly CMD="python ${SCRIPT_PATH}"

python -V

pip install moss-decoder --upgrade

cargo install hyperfine --locked

hyperfine \
    "${CMD}"\
    --warmup 3\
    --style full\
    --time-unit millisecond\
    --shell=bash\
    --export-markdown bench.md
