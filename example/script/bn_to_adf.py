from biodivine_aeon import *
import sys

"""
This script implements conversion of Boolean networks in the formats
supported by AEON (bnet, aeon, sbml) into a format supported by abstract
argumentation frameworks (see https://www.cs.helsinki.fi/group/coreo/k++adf/).

Usage:
python3 bn_to_adf.py path/to/bn_model.aeon > adf_file.txt
"""

bn = BooleanNetwork.from_file(sys.argv[1])

for var in bn.variables():
	print(f"s({int(var) + 1}).")	

def convert_function(function):
	if function.is_const():
		if function.as_const():
			return "c(v)"
		else:
			return "c(f)"
	if function.is_var():
		var = function.as_var()
		return str(int(var) + 1)
	if function.is_param():
		raise Error("Parameters not supported.")
	if function.is_not():
		inner = function.as_not()
		return f"neg({convert_function(inner)})"
	if function.is_and():
		(a, b) = function.as_and()
		return f"and({convert_function(a)}, {convert_function(b)})"
	if function.is_or():
		(a, b) = function.as_or()
		return f"or({convert_function(a)}, {convert_function(b)})"
	if function.is_xor():
		(a, b) = function.as_xor()
		return f"xor({convert_function(a)}, {convert_function(b)})"
	if function.is_imp():
		(a, b) = function.as_imp()
		return f"imp({convert_function(a)}, {convert_function(b)})"
	if function.is_iff():
		(a, b) = function.as_iff()
		return f"iff({convert_function(a)}, {convert_function(b)})"


for var in bn.variables():
	function = bn.get_update_function(var)
	print(f"ac({int(var) + 1}, {convert_function(function)}).")