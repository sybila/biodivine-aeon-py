# Biodivine `pbn-control` API coverage

Currently, the API of the control module has been quite substantially reworked, so the correspondence
between Rust and Python types is not always straightforward. In general, we introduced `PerturbationSet`, 
`PerturbationModel` and `ColoredPerturbationSet` which behave similar to the other symbolic structures
that already exist in AEON. This replaces structures like `AttractorControlMap` and `PerturbationControlMap`.

We also modified the `AsynchronousPerturbationGraph` such that it actually inherits from the `AsynchronousGraph`,
which reduces the amount of functions that need to be implemented separately. The most important part of this
effort is additional sanitization of the symbolic encoding where needed in order to "convince" the user that
in the default mode, the perturbation graph works just like the regular one.

There is still an API export here for the sake of completeness, but in the future the plan is to also
update the Rust version to better align with the Python API.