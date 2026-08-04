from biodivine_aeon import *
from pathlib import Path
import sys

# Takes BN path from the first argument:
bn_path = sys.argv[1]
bn = BooleanNetwork.from_file(bn_path)
bn = bn.infer_valid_graph()

if bn_path.endswith(".sbml"):
	Path(Path(bn_path).with_suffix(".fixed.sbml")).write_text(bn.to_sbml())
if bn_path.endswith(".aeon"):
	Path(Path(bn_path).with_suffix(".fixed.aeon")).write_text(bn.to_aeon())
if bn_path.endswith(".json"):
	Path(Path(bn_path).with_suffix(".fixed.json")).write_text(bn.to_bma_json())
if bn_path.endswith(".bnet"):
	print("BNET does not support regulation edge annotations. Saving as AEON.")
	Path(Path(bn_path).with_suffix(".fixed.aeon")).write_text(bn.to_aeon())
if "booleannet" in bn_path:
	print("BooleanNet does not support regulation edge annotations. Saving as AEON.")
	Path(Path(bn_path).with_suffix(".fixed.aeon")).write_text(bn.to_aeon())