# Biodivine/AEON.py

This repository contains AEON.py, the Python bindings for the AEON tool, which can be used for symbolic analysis of (partially specified) Boolean networks. In particular, it allows solving attractor detection and source-target control problems on large, non-trivial networks. Furthermore, these problems can be addressed even in networks with logical parameters or partially unknown dynamics.

### Installation

The package is available through `PyPI` for all major operating systems (Windows, Linux and macOS). To install it, you can simply run:

```
pip install biodivine-aeon
```

### Documentation

For new users, we recommend familiarising with the three Jupyter notebooks below, which summarise most of the major functionality of AEON.py:
 - [Symbolic computation using BDDs](https://deepnote.com/project/Aeonpy-Examples-CR33GbmyS2e4tqqZCcCwjA/%2Fexample_bdd.ipynb)
 - [Working with parametrised Boolean networks](https://deepnote.com/project/Aeonpy-Examples-CR33GbmyS2e4tqqZCcCwjA/%2Fexample_bn.ipynb)
 - [AEON.py example project](https://deepnote.com/project/Aeonpy-Examples-CR33GbmyS2e4tqqZCcCwjA/%2Fexample_aeon.ipynb)

Subsequently, the [manual](https://biodivine.fi.muni.cz/aeon/manual/v0.4.0/index.html) of the standalone AEON tool (which AEON.py is based on) can be also helpful to understand some of the high-level concepts.

More advanced users can inspect a detailed API documentation available [here](https://biodivine.fi.muni.cz/docs/aeon-py/v0.1.0/). Note that this is a documentation generated for the *Rust* codebase, which is then exported into Python using the `PyO3` tool. As such, note that some names may be different in the exported Python library (observe the `name` attribute on most structs that is used for this reason). Nevertheless, the documentation should describe all available methods and data structures.

### Building from source

At the moment, the build process includes a Z3 wrapper that will use your local Z3 installation. On linux, to build
this wrapper, you'll need `clang` (and `cmake` and other standard build tools). This is only required during build,
however, even at runtime, you might need to have Z3 installed to use certain features if you are running a locally
compiled version of `AEON.py`.

If you are building a "release" version, we recommend running the build with `--features static-z3` (see our CI 
scripts on how to do this). This will "bake in" the Z3 library into the final package, which takes a lot more 
time to build (~30min), but it means you don't need a local Z3 installation to run `AEON.py`. Also, the users 
cannot break the library by having an outdated or otherwise incompatible Z3 installed.

```
python3 -m pdoc --html --force docs/biodivine_aeon.py
```