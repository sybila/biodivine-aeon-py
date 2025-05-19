#!/bin/bash

# TODO: download and unzip the data files

if [ -z "$1" ]; then
  echo "Usage: $0 <timeout>"
  exit 1
fi

cd "$(dirname "$0")"

ulimit -v 2097152
timeout_val="$1"

scripts=(
  "fixed_points.py"
  "fixed_points_new.py"
  "reachability_fwd.py"
  "reachability_fwd_new.py"
  "reachability_bwd.py"
  "reachability_bwd_new.py"
  "minimal_trap_spaces.py"
  "minimal_trap_spaces_new.py"
)

for script in "${scripts[@]}"; do
  echo "Running $script with timeout $timeout_val..."
  python3 run.py "$timeout_val" edition-2022-aeon "$script"
done

echo "All scripts executed."

python report.py
echo "Report generated."

echo "Cleaning up..."
rm -rf _run_edition-2022-aeon_*

echo "Cleanup complete."

echo "To see the report, run:"
echo "cat report.txt"
