#!/bin/bash

set -ex

if [ `uname` == Darwin ]; then
  export HOME=`mktemp -d`
fi

maturin build --interpreter python --release

$PYTHON -m pip install target/wheels/*.whl --no-deps --ignore-installed -vv