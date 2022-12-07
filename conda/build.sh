#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  #  --universal2 (disabled for now)
  maturin build --release --interpreter python -o dist --features static-z3
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist --features static-z3
fi

pip install dist/*.whl --no-deps --ignore-installed