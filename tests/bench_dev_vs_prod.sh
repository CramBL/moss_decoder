#!/bin/bash

source ./tests/utils.sh
source ./tests/performance_dev_py.sh "just importing"
source ./tests/performance_prod_py.sh "just importing"

measure_performance_dev
measure_performance_prod

println_magenta "*** Benchmark concluded ***\n"

println_blue "Benchmark of development build"
println_bright_yellow "$( cat dev-bench.md )\n"

println_cyan "Benchmark of production build"
println_bright_yellow "$( cat prod-bench.md )\n"
