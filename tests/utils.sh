#!/bin/bash

export SCRIPT_PATH="tests/integration.py"
export BENCH_CMD="python ${SCRIPT_PATH} benchmark"

export TXT_CLEAR="\e[0m"

export TXT_YELLOW="\e[33m"
function println_yellow {
    printf  "\e[33m%b\e[0m\n" "${1}"
}

export TXT_BRIGHT_CYAN="\e[96m"
function println_cyan {
    printf "\e[96m%b\e[0m\n" "${1}"
}

export TXT_RED="\e[31m"
function println_red {
    printf "\e[31m%b\e[0m\n" "${1}"
}

export TXT_GREEN="\e[32m"
function println_green {
    printf "\e[32m%b\e[0m\n" "${1}"
}

export TXT_BRIGHT_GREEN="\e[92m"
function println_bright_green {
    printf "\e[92m%b\e[0m\n" "${1}"
}

export TXT_BLUE="\e[34m"
function println_blue {
    printf "\e[34m%b\e[0m\n" "${1}"
}

export TXT_BRIGHT_MAGENTA="\e[95m"
function println_magenta {
    printf "\e[95m%b\e[0m\n" "${1}"
}

export TXT_BRIGHT_YELLOW="\e[93m"
function println_bright_yellow {
    printf "\e[93m%b\e[0m\n" "${1}"
}
