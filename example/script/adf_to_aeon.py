from pathlib import Path
from biodivine_aeon import *
import sys

"""
This script convers an ADF file (abstract argumentation frameworks, see 
https://www.cs.helsinki.fi/group/coreo/k++adf/) into an AEON Boolean network. 

If you want the output to be in different format (bnet, SBML), you can change 
the last line to `to_bnet()` or `to_sbml()` instead of `to_aeon()`.

Usage example:
python3 adf_to_aeon.py path/to/adf_model.txt > model.aeon
"""

file_content = Path(sys.argv[1]).read_text()

# Need to replace these to avoid keyword clashes.
file_content = file_content.replace("not", "f_not")
file_content = file_content.replace("and", "f_and")
file_content = file_content.replace("or", "f_or")
file_content = file_content.replace("imp", "f_imp")
file_content = file_content.replace("xor", "f_xor")
file_content = file_content.replace("iff", "f_iff")
file_content = file_content.replace("c(v)", "BooleanExpression.mk_const(True)")
file_content = file_content.replace("c(f)", "BooleanExpression.mk_const(False)")

variables = {}
functions = {}

def s(x):
	variables[x] = f"v_{x}"

def ac(x, expr):
	if isinstance(expr, int):
		expr = BooleanExpression.mk_var(f"v_{expr}")
	functions[x] = expr

def f_not(expr):
	if isinstance(expr, int):
		expr = BooleanExpression.mk_var(f"v_{expr}")
	return BooleanExpression.mk_not(expr)

def f_and(a, b):
	if isinstance(a, int):
		a = BooleanExpression.mk_var(f"v_{a}")
	if isinstance(b, int):
		b = BooleanExpression.mk_var(f"v_{b}")
	return BooleanExpression.mk_and(a, b)

def f_or(a, b):
	if isinstance(a, int):
		a = BooleanExpression.mk_var(f"v_{a}")
	if isinstance(b, int):
		b = BooleanExpression.mk_var(f"v_{b}")
	return BooleanExpression.mk_or(a, b)

def f_imp(a, b):
	if isinstance(a, int):
		a = BooleanExpression.mk_var(f"v_{a}")
	if isinstance(b, int):
		b = BooleanExpression.mk_var(f"v_{b}")
	return BooleanExpression.mk_imp(a, b)

def f_xor(a, b):
	if isinstance(a, int):
		a = BooleanExpression.mk_var(f"v_{a}")
	if isinstance(b, int):
		b = BooleanExpression.mk_var(f"v_{b}")
	return BooleanExpression.mk_xor(a, b)

def f_iff(a, b):
	if isinstance(a, int):
		a = BooleanExpression.mk_var(f"v_{a}")
	if isinstance(b, int):
		b = BooleanExpression.mk_var(f"v_{b}")
	return BooleanExpression.mk_iff(a, b)

for line in file_content.split('\n'):	
	if line.strip() == "":
		continue	
	eval(line)

bn = BooleanNetwork(sorted(variables.values()))
for var, fun in functions.items():
	var = f"v_{var}"
	regulators = fun.support_set()
	for reg in regulators:
		bn.ensure_regulation({
			'source': reg,
			'target': var,
			'sign': None,
			'essential': False
		})

	fun = UpdateFunction(bn, fun)
	bn.set_update_function(var, fun)

bn = bn.infer_valid_graph()
print(bn.to_aeon())