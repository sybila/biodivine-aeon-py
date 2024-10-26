This folder contains documents which outline how the` biodivine` Rust API 
is mapped to the AEON.py API. Specifically, for every Rust library, we have a text file
(`*.api.txt`) which is the output of
[cargo-public-api](https://github.com/Enselic/cargo-public-api), and a
Markdown document which outlines how the public API maps to Python
classes/methods.

 > When using `cargo public-api`, add the `-sss` argument to generate a
 > simplified summary. 

When you update the Rust dependency, you should:

- Generate a new public API text file (or download it from the release
  section of the relevant library---this is not supported right now, but
  the goal is to append public API automatically to every release).
- Compare the diff between the current `*.api.txt` and your new file.
- Add the changes to the Markdown document: not everything has to be
  implemented in Python, but every change should be reflected in Markdown.
- Trivial items (like `Debug`, automatic conversions, etc.) don't have to
  be included in the Markdown comparison, but anything that represents
  non-trivial Rust functionality should be mentioned.
- Please make sure to include "special" Python methods for implementing 
  things like equality or ordering, as PyO3 does not generate these 
  automatically, even if the original structures implement the respective traits. 
  See [Python data model](https://docs.python.org/3/reference/datamodel.html) 
  for more details. 
- You can also specify `__copy__` and `__deepcopy__` methods
  that should work with the default Python `copy` module.
- Finally, under ideal conditions, we would like to ensure that most types
  are [pickle-friendly](https://docs.python.org/3/library/pickle.html). Usually,
  this can be achieved by implementing `__getstate__` and `__setstate__` (see
  [here](https://gist.github.com/ethanhs/fd4123487974c91c7e5960acc9aa2a77)).
- Implement the changes within the Rust crate. The Rust documentation will
  be used for Python too, so keep that in mind (e.g. use Python examples).
- Add type annotations for the updated methods into `biodivine_aeon/__init__.pyi`.
  If you need to create a type alias, add it to `biodivine_aeon/__init__.py` too.

> Remember to use `--features solver-z3` when building the public API for
> `lib-param-bn`.

Some notes about writing bindings:
  - For argument types, use `Sequence` and `Mapping` instead of `list` nad `dict`.
  - For conversion methods that create a copy/new item, use `to_*`.
  - For conversion methods that return a reference to some inner python object, use `get_` or no prefix.