# Biodivine/AEON.py

AEON.py provides Python bindings for the internal and experimental functionality of the tool
[AEON](https://biodivine.fi.muni.cz/aeon/). You can use it to perform analysis of 
Boolean networks with symbolic methods. In particular, AEON.py supports:

 - Classical and *partially specificed* Boolean networks (i.e. with missing or partially unknown update functions).
 - Major network formats like `.sbml` and `.bnet`, including model validation.
 - Competitive attractor detection methods using binary decision diagrams (BDDs).
 - Competitive fixed-point enumeration methods.
 - Basic control/reprogramming approaches.
 - Arbitrary symbolic operations on sets of Boolean states/functions represented through BDDs.
 - Symbolic model checking of HCTL properties.

### Installation

The package is available through `PyPI` for all major operating systems (Windows, Linux and macOS). 
To install it, you can simply run:

```
pip install biodivine-aeon
```

AEON.py is also available through [conda](https://anaconda.org/daemontus/biodivine_aeon) 
and the [CoLoMoTo Docker](https://github.com/colomoto/colomoto-docker) environment.

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
 - There are several "workflow" examples. Some are focused on a specific task (e.g. attractor
 or fixed-point detection) while others provide a general "overview" of a particular topic (
 like BDDs and symbolic algorithms in general).

Additionally, the 
[manual](https://biodivine.fi.muni.cz/aeon/manual/v0.4.0/index.html) of the standalone AEON tool
can be helpful to understand some of the high-level concepts related
to Boolean networks (both classical and partially specified).

Finally, more informed users can inspect a detailed API documentation available 
[here](https://biodivine.fi.muni.cz/docs/aeon-py/v0.1.0/). Note that this is a documentation generated 
for the *Rust* codebase, which is then exported into Python using the `PyO3` tool. 
As such, naming can be different in the exported Python library (observe the `name` 
attribute on most structs that is used for this reason). Nevertheless, the documentation should 
describe all available methods and data structures.

> A proper Python documentation of the full library API is planned later for the `1.0.0` release.
> Until then, you may also inspect the internal documentation of the underlying Rust libraries
> to see what functionality is generally available: 
> [lib-bdd](https://docs.rs/biodivine-lib-bdd/0.5.1/biodivine_lib_bdd/), 
> [lib-param-bn](https://docs.rs/biodivine-lib-param-bn/0.4.5/biodivine_lib_param_bn/).

## Development instructions

To build AEON.py from source, you generally need to follow the guides/instructions available for the
[maturin](https://github.com/PyO3/maturin) tool. However, since some of the functionality in AEON.py
requires the Z3 solver, the process is slightly more error prone as it also involves C dependencies,
not only pure Rust (this also complicates builds on Apple Silicon and more exotic CPUs).

### Local builds

To build and test AEON.py locally, you can generally follow the official instructions for building
packages using `maturin`. However, you have two options for integrating with Z3: either as a static
or as a dynamic library. 

 - Using static integration is more "stable" since the library will use a known 
   version of Z3 tested by us. However, Z3 will need to be built during the first
   compilation, which can take ~30min (subsequent builds should be faster thanks
   to build cache). You can also encounter build errors if there are issues with
   your C/C++ toolchain. To use the static linking method, you'll need to add
   extra commandline arguments when building the library (see below).
 - Dynamic integration uses the version of Z3 installed on your system. As such,
   the compilation is faster since there's no need to build Z3. However, we do not
   guarantee that your installed version is compatible. Furthermore, you'll need to
   make sure your version is installed in such a way that it can be used as a dynamic
   library (the `.h` and `.so/.dylib/.dll` files are available in their respective
   include paths). Ideally, to use this method, you should only need to install Z3 on
   your system using the official method (e.g. `apt install z3`, `brew install z3`, or
   use the official windows installer).

In general, we recommend starting with dynamic linking, because if everything works, it is faster
and easier. However, in case you run into trouble, static linking could be actually easier
to debug, since it depends less on your actual configuration and is thus easier to reproduce. 
Similarly, it can be easier to use static linking on systems where Z3 is not available through
an official installer or cannot be installed globally.

 > In any case, on linux, you'll need typical "essential" build tools like `cmake` and `clang`
 > to even build the Z3 dependency, regardless of the linking process. On debian-ish distros,
 > `apt install build-essential clang` should be sufficient.

To install a local version of AEON.py, you then simply need to follow the same steps outlined 
in the `maturin` [tutorial](https://www.maturin.rs/tutorial):

 - Install `maturin` (see [here](https://www.maturin.rs/installation)).
 - Create a Python [virtual environment](https://docs.python.org/3/library/venv.html) for testing and activate it.
 - \[Dynamic linking\] Run `maturin develop` to install a local version of AOEN.py into this virtual environment.
 - \[Static linking\] Run `maturin develop --features static-z3` to do the same, but with a static version of Z3.

If the build passes, you should be able to use the library on your local machine. Feel free to also install jupyter
notebooks and test the library in the interactive environment (or on one of the examples).

#### Publishing

Finally, you may want to release an alpha/beta version of the library to test that everything is working correctly
on all platforms (builds are notoriously finicky in these situations, since we essentially have to build for
multiple platforms and multiple versions of Python). Fortunately, the CI is set up to automatically build 
and publish the library on all relevant platforms every time a new tag is pushed. 

Before you publish a new version, make sure that the build works at least on your own machine. Then, make sure to update
the library versions in all build files. Specifically, you should update the version in `pyproject.toml` (publishing 
on PyPI), `cargo.toml` (Rust crate version, not published at the moment), and `conda/meta.yml` (publishing on Anaconda). 

Not everything is relevant for every publishing method, but it is generally a good idea to update all files to ensure
consistency. For `pyproject.toml` and `conda/meta.yml`, you can use suffix `aX` to indicate that the version is
an "alpha" version (e.g. `0.4.0a2`). In `Cargo.toml`, you have to use `-alphaX` instead (e.g. `0.4.0-alpha2`).

Finally, either create a new git tag and push it, or create a new github release with the new tag. Ideally, the tag 
should be equivalent to the Rust crate version (e.g. `0.4.0-alpha2`). 

 > If the build fails and you want to fix it, you can actually reuse the same tag: Once you've made the changes, delete
   the tag locally and push the change (this may need a force push, but since you are the only person using this tag,
   it should be ok). Then create the tag again and push it again. It should be also possible to
   [overwrite](https://stackoverflow.com/questions/25815631/git-force-push-tag-when-the-tag-already-exists-on-remote)
   the tag directly.

Once everything is working as expected, you can remove the `alpha` suffixes and properly release a new version (in which
case, please include a detailed changelog in the release description on github).
