#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then  
  # For whatever reason, I cannot seem to successfully install the arm target such that maturin can use it...
  # rustup target add aarch64-apple-darwin
  # --target universal2-apple-darwin
  maturin build --release --interpreter python -o dist --target universal2-apple-darwin
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist
fi

pip install dist/*.whl --no-deps --ignore-installed