# Biodivine/Aeon Python Bindings

This repository contains Python bindings for the Biodivine Rust libraries and tools. Each sub-folder contains one library, with an additional readme and more detailed instructions.

If you want to quickly explore the supported features, you can have a look at these [jupyter notebooks](https://deepnote.com/project/Aeonpy-Examples-CR33GbmyS2e4tqqZCcCwjA/%2Fexample_bn.ipynb).

> Note that due to the way native Python bindings are implemented, there is no interoperability between the three libraries in this repository (i.e. you can't use a BDD from `lib-bdd` in `aeon`, etc.). Instead, each library has its own copy of the data-types from the lower-level libraries (i.e. `lib-param-bn` re-exports `lib-bdd` and `aeon` re-exports both `lib-bdd` and `lib-param-bn`). The advantage is that you don't need to import multiple libraries, `from biodivine_aeon import *` should be sufficient in most use cases. 