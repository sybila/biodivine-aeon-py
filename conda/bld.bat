maturin build --release --no-sdist -o dist
dir dist
pip install "dist/*.whl"