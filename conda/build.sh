#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  maturin build --release --interpreter python -o dist
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist
fi

pip install dist/*.whl --no-deps --ignore-installed