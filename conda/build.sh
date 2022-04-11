#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  maturin build --release --interpreter python --no-sdist -o dist #--universal2    
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist
fi

pip install dist/*.whl --no-deps --ignore-installed