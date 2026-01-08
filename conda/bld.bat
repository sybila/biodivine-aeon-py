maturin list-python
maturin build --release --interpreter python -o dist
dir dist
pip install dist/*.whl --no-deps --ignore-installed