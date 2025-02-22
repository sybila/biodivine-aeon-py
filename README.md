# Fork information

> This part of the README documents fork changes, regular documentation
> continues below.

To test the changes made so far, run
`python example/script/algorithms.py edition-2022-aeon/007.aeon`.

# Biodivine/AEON.py

 > AEON.py now finally has API documentation! You can learn more about the individual methods and classes 
 > [here](https://biodivine.fi.muni.cz/docs/aeon-py/latest/). 

AEON.py provides Python bindings for the internal and experimental functionality of the tool
[AEON](https://biodivine.fi.muni.cz/aeon/). You can use it to perform analysis of 
Boolean networks with symbolic (BDD-based or solver-based) methods. In particular, AEON.py supports:

 - Classical and *partially specified* Boolean networks (i.e. with missing or partially unknown update functions).
 - Major network formats like `.sbml` and `.bnet`, including model validation.
 - Competitive symbolic methods for:
    - Attractor detection.
    - Fixed-point enumeration.
    - Minimal/maximal trap space enumeration.
 - Symbolic (H)CTL model checking and parameter synthesis.
 - Control/reprogramming methods.
 - Arbitrary symbolic operations on sets of Boolean states/space/functions represented through BDDs.

### Installation

The package is available through `PyPI` for all major operating systems (Windows, Linux and macOS). 
To install it, you can simply run:

```
pip install biodivine_aeon
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

 > We also provide some simple scripts for performing common tasks in the `example/scripts` folder. However, these
 > by far do not cover everything that is supported by AEON.py. 

The documentation of the AEON.py API is available [here](https://biodivine.fi.muni.cz/docs/aeon-py/latest/).
It should describe the functionality of all the classes/methods, but it currently does not 
provide self-contained examples.

For that, we recommend the Jupyter notebooks available in the `examples` directory:
 - There are three non-trivial case studies using AEON.py for analysing attractor and phenotype
 bifurcations in real-world Boolean networks.
 - There are several "workflow" examples. Some are focused on a specific task (e.g. attractor
 or fixed-point detection) while others provide a general "overview" of a particular topic (
 like BDDs and symbolic algorithms in general).

Additionally, the 
[manual](https://biodivine.fi.muni.cz/aeon/manual/v0.4.0/index.html) of the standalone AEON tool
can be helpful to understand some of the high-level concepts related
to partially specified Boolean networks (both classical and partially specified).

## Development instructions

To build AEON.py from source, you generally need to follow the guides/instructions available for the
[maturin](https://github.com/PyO3/maturin) tool. However, since some of the functionality in AEON.py
requires the Z3 solver, the process is slightly more error-prone, as it also involves C dependencies,
not only pure Rust (this also complicates builds on Apple Silicon and more exotic CPUs).

### Local builds

To build and test AEON.py locally, you can generally follow the official instructions for building
packages using `maturin`. However, you have two options for integrating with Z3: either as a static
or as a dynamic library. 

 - Using static integration is more "stable" since the library will use a known 
   version of Z3 tested by us. However, Z3 will need to be built during the first
   compilation, which can take ~30min (subsequent builds should be faster thanks
   to the build cache). You can also encounter build errors if there are issues with
   your C/C++ toolchain. To use the static linking method, you'll need to add
   extra commandline arguments when building the library (see below).
 - Dynamic integration uses the version of Z3 installed on your system. As such,
   the compilation is faster since there's no need to build Z3. However, we do not
   guarantee that your installed version is compatible. Furthermore, you'll need to
   make sure your version is installed in such a way that it can be used as a dynamic
   library (the `.h` and `.so/.dylib/.dll` files are available in their respective
   include paths). Ideally, to use this approach, you should only need to install Z3 on
   your system using the official method (e.g. `apt install z3`, `brew install z3`, or
   use the official Windows installer).

In general, we recommend starting with dynamic linking, because if everything works, it is faster
and easier. However, in case you run into trouble, static linking could be actually easier
to debug, since it depends less on your actual configuration and is thus easier to reproduce across 
different machines. Similarly, it can be easier to use static linking on systems where Z3 is not 
available through an official installer or cannot be installed globally.

 > In any case, on linux, you'll need typical "essential" build tools like `cmake` and `clang`
 > to even build the Z3 dependency, regardless of the linking process. On debian-ish distros,
 > `apt install build-essential cmake clang` should be sufficient.

 > On Apple Silicon, dynamic linking for Z3 is currently not working out-of-the-box if
 > you installed Z3 through `brew`, because the library files are not discoverable by `clang`
 > by default. To fix this issue, you need to update `CPATH` and `LIBRARY_PATH` (use correct
 > Z3 location based on your installed version):
 > ```bash
 > export CPATH=$CPATH:/opt/homebrew/Cellar/z3/4.12.2/include          
 > export LIBRARY_PATH=$LIBRARY_PATH:/opt/homebrew/Cellar/z3/4.12.2/lib
 > ```

To install a local version of AEON.py, you then simply need to follow the same steps outlined 
in the `maturin` [tutorial](https://www.maturin.rs/tutorial):

 - Install `maturin` (see [here](https://www.maturin.rs/installation)).
 - Create a Python [virtual environment](https://docs.python.org/3/library/venv.html) for testing and activate it.
 - \[Dynamic linking\] Run `maturin develop` to install a local version of AEON.py into this virtual environment.
 - \[Static linking\] Run `maturin develop --features static-z3` to do the same, but with a static version of Z3.

If the build passes, you should be able to use the library on your local machine. Feel free to also install Jupyter
notebooks and test the library in the interactive environment (or on one of the examples).

#### Other tasks

After successfully running `maturin develop`, you can use `pytest ./tests` to execute a set of Python unit tests.
Code coverage can be computed for these tests based on the [official example](https://github.com/cjermain/rust-python-coverage).
Basic type integrity of the tests can be also validated through `mypy tests`.

Similarly, you can generate documentation using `pdoc` by running `python3 -m pdoc biodivine_aeon`. This combines 
documentation in Rust comments with type aliases in `biodivine_aeon/__init__.py`.

### Upgrading dependencies

Currently, there is no automated way of generating bindings for newly created (or updated) methods. As such, when
upgrading a `biodivine` dependency, you should make sure to create/update bindings for all relevant methods.

As this is a rather error-prone process, we provide documentation that we use to track relevant API changes.
You can find this documentation in the `api-coverage` folder. Please follow the instructions given in the 
`README` of this folder when upgrading a library dependency.

Furthermore, once you update the library bindings, you also need to manually update the python documentation.
This documentation is stored in the `api-docs` folder.

### Publishing

Finally, you may want to release an alpha/beta version of the library to test that everything is working correctly
on all platforms (builds are notoriously finicky in these situations, since we essentially have to build for
every platform and multiple versions of Python). Fortunately, the CI is set up to automatically build 
and publish the library on all relevant platforms every time a new tag is pushed. 

Before you publish a new version, make sure that the build works at least on your own machine. Then, make 
sure to update the library versions in all the build files. Specifically, you should update the version in 
`pyproject.toml` (publishing on PyPI), `cargo.toml` (Rust crate version, not published at the moment), and 
`conda/meta.yml` (publishing on Anaconda). 

Not everything is relevant for every publishing method, but it is generally a good idea to update all files to ensure
consistency. For `pyproject.toml` and `conda/meta.yml`, you can use suffix `aX` to indicate that the version is
an "alpha" version (e.g. `0.4.0a2`). In `Cargo.toml`, you have to use `-alphaX` instead (e.g. `0.4.0-alpha2`).

Finally, either create a new git tag and push it, or create a new GitHub release with the new tag. Ideally, the tag 
should be equivalent to the Rust crate version (e.g. `0.4.0-alpha2`). 

 > If the build fails, and you want to fix it, you can actually reuse the same tag: Once you've made the changes, 
 > delete the tag locally and push the change (this may need a force push, but since you are the only person using 
 > this tag, it should be ok). Then create the tag again and push it again. It should be also possible to 
 > [overwrite](https://stackoverflow.com/questions/25815631/git-force-push-tag-when-the-tag-already-exists-on-remote) 
 > the tag directly.

Once everything is working as expected, you can remove the `alpha` suffixes and properly release a new version (in 
which case, please include a detailed changelog in the release description on GitHub).
