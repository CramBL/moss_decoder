#!/bin/bash

source ./tests/utils.sh
source ./tests/performance_dev_py.sh "just importing"
source ./tests/performance_prod_py.sh "just importing"

measure_performance_dev
measure_performance_prod

println_magenta "*** Benchmark concluded ***\n"

println_blue "Benchmark of development build"
cat dev-bench.md
println_yellow "---------"

println_cyan "Benchmark of production build"
cat prod-bench.md
println_yellow "---------"
