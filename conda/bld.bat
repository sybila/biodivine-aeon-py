maturin build --release -o dist
dir dist
pip install "dist/*.whl"