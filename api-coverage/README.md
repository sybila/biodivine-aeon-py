This folder contains documents which outline how the Rust API is mapped to
the Python API. Specifically, for every Rust library, we have a text file 
(`*.api.txt`) which is the output of 
[cargo-public-api](https://github.com/Enselic/cargo-public-api), and a 
Markdown document which outlines how the public API maps to Python 
classes/methods.

When you update the Rust dependency, you should:

 - Generate a new public API text file.
 - Compare the diff between the current `*.api.txt` and your new file.
 - Add the changes to the Markdown document: not everything has to be 
   implemented in Python, but every change should be reflected in Markdown.
 - Trivial items (like `Debug`, automatic conversions, etc.) don't have to 
   be included in the Markdown comparison, but anything that represents 
   non-trivial Rust functionality should be mentioned.
 - Ideally implement everything in Python :)
