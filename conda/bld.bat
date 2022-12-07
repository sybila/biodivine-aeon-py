maturin build --release --features static-z3 -o dist
dir dist
pip install "dist/*.whl"