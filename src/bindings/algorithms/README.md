# Module information

To compile AEON.py with the new algorithms bindings, proceed normally with the installation, but add the following
flag to the `maturin develop` command:

```
maturin develop --features algorithms-pyo3-bindings
```

Then, you can run individual scripts from the `example/algorihtms_evaluation` folder to test the changes. To interupt
the computation, use Ctrl + C.

To run the benchmark, use the `/example/algorithms_evaluation/run.sh <timeout (e.g. 10s, 5m, 1h)>` command. The timeout is the maximum
time in seconds for each individual run. The script will run the benchmark for all algorithms
and all models in the `/example/algorithms_evaluation` folder. The results will be saved in the `/example/algorithms_evaluation/report.txt` file.

For a quick test, it is suggested for the timepout to be set to up to 10 seconds.
Please note that due to the short timeout, old implementations of the algorithms can fail to finish while
the new succeed and vice versa.

To interact with the new algorithms, you can use the `/example/algorithms_evaluation/evaluation.ipynb` Jupyter notebook.
To use it, install Jupyter:

```
pip install jupyter
```

Then, run it with the command:

```
jupyter notebook example/algorithms_evaluation/evaluation.ipynb
```

