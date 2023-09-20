# AEON.py examples

Here, we give several examples on how to use AEON.py to solve problems related to Boolean networks.
Currently, the examples are split into workflows and case studies.

Each workflow shows how to solve a particular problem using AEON.py. Each case study focuses on a specific
Boolean model and tries to obtain some useful information about it.

### Getting started

To be able to execute all examples, you'll need AEON.py and Jupyter notebooks. Optionally, some 
notebooks also use graphviz and pandas for visualisation:  

```
# Install using the attached requirements file.
pip install -r requirements.txt
# Alternatively, install everything using a single command:
pip install biodive_aeon notebook graphviz pandas
```

 > If you run into any issues, make sure that your version of AEON.py is up-to-date.

Afterward, you can start the Jupyter notebook environment:

```
jupyter notebook
```

This will print a URL into the command line which you can open to access the Jupyter 
environment with the example notebooks.

## Workflows

 - `workflow/manipulating-binary-decision-diagrams.ipynb` Many algorithms in AEON.py rely 
internally on binary decision diagrams (BDDs) when representing large sets. Normally, you won't 
need to interact with BDDs directly, but in case you need to, here is where you learn how to.
 - `workflow/manipulating-boolean-networks.ipynb` Here, you can learn how to load/store Boolean
networks from various formats or how to construct them directly in Python. Furthermore, the notebook
shows how to work with the symbolic representation of the network's state-transition graph.
 - `workflow/computing-fixed-points.ipynb` Shows how to use the symbolic and solver algorithms in
AEON.py to enumerate fixed-points of a particular Boolean network.
 - `workflow/computing-attractors.ipynb` Shows how to use the symbolic algorithms in AEON.py
to enumerate the asynchronous attractors of a particular Boolean network.
- `workflow/model-checking.ipynb` Shows how to run the model-checking algorithm in AEON.py
  to analyze temporal properties of Boolean networks.

## Case studies

 - `case-study/butanol-production/main.ipynb` Demonstrates how AEON.py can be used to repair a potential
logical error in a model of butanol production in clostridium bacteria.
 - `case-study/t-cell-signalling/main.ipynb` Analysis of a signalling model within leukemia cells. Shows
phenotype bifurcations with respect to model inputs.
 - `case-study/interferon-pathway/main.ipynb` Bifurcation analysis of the inflammation and immune response 
phenotypes with respect to the model inputs. 