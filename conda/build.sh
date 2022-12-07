#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  maturin build --release --interpreter python -o dist --universal2 --features z3
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist --features z3
fi

pip install dist/*.whl --no-deps --ignore-installed