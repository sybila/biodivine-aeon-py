#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <timeout (e.g., 10s, 1m, 1h)>"
  exit 1
fi

cd "$(dirname "$0")"

TARGET_DIR="edition-2022-aeon"
ZIP_FILE="edition-2022-aeon.zip"
URL="https://github.com/sybila/biodivine-boolean-models/releases/download/august-2022/$ZIP_FILE"

if [ ! -d "$TARGET_DIR" ]; then
  echo "Directory '$TARGET_DIR' does not exist. Downloading and extracting..."

  curl -L -o "$ZIP_FILE" "$URL"

  unzip "$ZIP_FILE"

  rm "$ZIP_FILE"

  echo "Done."
else
  echo "Directory '$TARGET_DIR' already exists. Skipping download."
fi

ulimit -v 8388608
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
  "percolation.py"
  "percolation_new.py"
)

for script in "${scripts[@]}"; do
  echo "Running $script with timeout $timeout_val..."
  python3 run.py "$timeout_val" "$TARGET_DIR" "$script"
done

echo "All scripts executed."

python report.py
echo "Report generated."

echo "Cleaning up..."
rm -rf _run_edition-2022-aeon_*

echo "Cleanup complete."

echo "To see the report, run:"
echo "cat report.txt"
