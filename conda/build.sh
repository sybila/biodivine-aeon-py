#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then  
  export MACOSX_DEPLOYMENT_TARGET="11.0"
  export CMAKE_OSX_DEPLOYMENT_TARGET="11.0"
  rustup target add aarch64-apple-darwin
  maturin build --target universal2-apple-darwin --release --interpreter python -o dist --features static-z3
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist --features static-z3
fi

pip install dist/*.whl --no-deps --ignore-installed