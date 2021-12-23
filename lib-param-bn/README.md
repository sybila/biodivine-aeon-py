# Biodivine (Parametrised) Boolean Networks (Python)

This repository hosts the Python bindings for the Biodivine (Parametrised) Boolean Networks library. This library provides support for performing basic operations with Boolean networks and their regulatory graphs. However, it has two important features that are (at the moment of writing) unique:

 - It has full support for symbolic exploration of the asynchronous state-transition graphs, which can be used to effectively perform exhaustive state space search (reachability, SCC detection, etc.) in very large networks (`2^100` states and similar).
 - It supports partially defined, or parametrised networks, meaning the update functions of the network can contain uninterpreted Boolean functions which serve as a placeholder for unknown network dynamics.

The library is available on `PyPi`, so you can install it directly through pip:

```
pip install biodivine_boolean_networks
```

A basic example of usage for the Python byndings is provided in this [Jupyter notebook](https://deepnote.com/project/Aeonpy-Examples-CR33GbmyS2e4tqqZCcCwjA/%2Fexample_bn.ipynb). To learn more details about the library, please refer to the original [Rust repository](https://github.com/sybila/biodivine-lib-param-bn) where you can find detailed documentation about its capabilities. Also, the documentation comments in this repository may be helpful to understand how some elements were translated from Rust to Python.


