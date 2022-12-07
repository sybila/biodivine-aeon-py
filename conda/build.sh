#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  #  --universal2 (disabled for now)
  export MACOSX_DEPLOYMENT_TARGET="11.0"
  export CMAKE_OSX_DEPLOYMENT_TARGET="11.0"
  maturin build --release --interpreter python -o dist --features static-z3
fi

if [ `uname` == Linux ]; then
  maturin build --release --interpreter python -o dist --features static-z3
fi

pip install dist/*.whl --no-deps --ignore-installed