# Biodivine/AEON.py

This repository contains AEON.py, the Python bindings for the AEON tool, which can be used for 
symbolic analysis of (partially specified) Boolean networks. In particular, it allows solving 
attractor detection and source-target control problems on large, non-trivial networks. Furthermore, 
these problems can be addressed even for networks with logical parameters or partially unknown dynamics.

### Installation

The package is available through `PyPI` for all major operating systems (Windows, Linux and macOS). 
To install it, you can simply run:

```
pip install biodivine-aeon
```

AEON.py is also available through `conda` and [CoLoMoTo Docker](https://github.com/colomoto/colomoto-docker).

### Citation

If you used AEON.py for some academic work, we'd be very happy if you could cite it using 
the following publication:

```
Beneš, N., Brim, L., Huvar, O., Pastva, S., Šafránek, D., & Šmijáková, E. (2022). 
AEON. py: Python library for attractor analysis in asynchronous Boolean networks. 
Bioinformatics, 38(21), 4978-4980.
```

### Documentation

> While AEON.py is relatively mature, as with many academic tools, there are still aspects of the
> documentation that are not completely finalized. If you find that something is missing, or just 
> want us to give you a demo of what the tool is capable of, feel free to get in touch!

For new users that are already familiar with the concept of Boolean networks, we recommend the
Jupyter notebooks available in the `examples` directory:
 - There are three non-trivial case studies using AEON.py for analysing attractor and phenotype
 bifurcations in real-world Boolean networks.
 - There are several "workflow" examples. Some are focused on a specific task (like attractor
 or fixed-point detection) while others provide a general "overview" of a particular topic (
 like BDDs and symbolic algorithms in general).

Additionally, the 
[manual](https://biodivine.fi.muni.cz/aeon/manual/v0.4.0/index.html) of the standalone AEON tool 
(which AEON.py is based on) can be helpful to understand some of the high-level concepts related
to Boolean networks (with and without parameters).

Finally, more advanced users can inspect a detailed API documentation available 
[here](https://biodivine.fi.muni.cz/docs/aeon-py/v0.1.0/). Note that this is a documentation generated 
for the *Rust* codebase, which is then exported into Python using the `PyO3` tool. 
As such, note that some names may be different in the exported Python library (observe the `name` 
attribute on most structs that is used for this reason). Nevertheless, the documentation should 
describe all available methods and data structures.

> A proper Python documentation of the full library API is planned later for the `1.0.0` release.
> Until then, you may also inspect the internal documentation of the underlying Rust libraries
> to see what functionality is generally available: 
> [lib-bdd](https://docs.rs/biodivine-lib-bdd/0.5.1/biodivine_lib_bdd/), 
> [lib-param-bn](https://docs.rs/biodivine-lib-param-bn/0.4.5/biodivine_lib_param_bn/).

### Building from source

At the moment, the build process includes a Z3 wrapper that will use your local Z3 installation. On linux, to build
this wrapper, you'll need `clang` (and `cmake` and other standard build tools). This is only required during build,
however, even at runtime, you might need to have Z3 installed to use certain features if you are running a locally
compiled version of `AEON.py`. That is, the default build links Z3 dynamically.

If you are building a "release" version, we recommend running the build with `--features static-z3` (see our CI 
scripts on how to do this). This will link the Z3 library statically into the final package, which takes a lot more 
time to build (~30min), but it means you don't need a local Z3 installation to run `AEON.py`. Also, the users 
cannot break the library by having an outdated or otherwise incompatible Z3 installation.