# Biodivine `lib-bdd` API coverage

This document should be up-to-date with `lib-bdd` version `0.5.13`.

## `BddVariable` (frozen)

<!--suppress XmlDeprecatedElement, HtmlDeprecatedAttribute -->
<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddVariable::from_index</code></td>
            <td><code>BddVariable.__init__</code></td>
        </tr>
        <tr>
            <td><code>BddVariable::eq</code></td>
            <td rowspan="2"><code>BddVariable.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>BddVariable::cmp</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariable.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariable.__repr__</code></td>
        </tr>
        <tr>
            <td><code>BddVariable::hash</code></td>
            <td><code>BddVariable.__hash__</code></td>
        </tr>
        <tr>
            <td><code>BddVariable::to_index</code></td>
            <td><code>BddVariable.__index__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariable.__getnewargs__</code></td>
        </tr>
    </tbody>
</table>

## `BddPointer` (frozen)

<!--suppress XmlDeprecatedElement, HtmlDeprecatedAttribute -->
<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddPointer::from_index</code></td>
            <td rowspan="2"><code>BddPointer.__init__</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::from_bool</code></td> 
        </tr>
        <tr>
            <td><code>BddPointer::eq</code></td>
            <td rowspan="2"><code>BddPointer.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::cmp</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>BddPointer.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPointer.__repr__</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::hash</code></td>
            <td><code>BddPointer.__hash__</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::to_index</code></td>
            <td><code>BddPointer.__index__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPointer.__getnewargs__</code></td>
        </tr> 
        <tr><td colspan="2" align="center">Utilities</td></tr>
        <tr>
            <td><code>BddPointer::zero</code></td>
            <td><code>BddPointer.zero</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::one</code></td>
            <td><code>BddPointer.one</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::is_zero</code></td>
            <td><code>BddPointer.is_zero</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::is_one</code></td>
            <td><code>BddPointer.is_one</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::is_terminal</code></td>
            <td><code>BddPointer.is_terminal</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::as_bool</code></td>
            <td><code>BddPointer.as_bool</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::zero</code></td>
            <td><code>BddPointer.zero</code></td>
        </tr>
        <tr>
            <td><code>BddPointer::flip_if_terminal</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

## `BddNode`

We are ignoring `BddNode` for now, because it seems a bit too low level 
for the current scope of features in AEON.py. But it might be useful to
explore it in the future.

## `BddVariableSetBuilder`

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddVariableSetBuilder::new</code></td>
            <td><code>BddVariableSetBuilder.__init__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__getstate__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSetBuilder.__setstate__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Builder methods</td></tr>
        <tr>
            <td><code>BddVariableSetBuilder::make_variable</code></td>
            <td><code>BddVariableSetBuilder.add</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSetBuilder::make</code></td>
            <td rowspan="2"><code>BddVariableSetBuilder.add_all</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSetBuilder::make_variables</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSetBuilder::build</code></td>
            <td><code>BddVariableSetBuilder.build</code></td>
        </tr>        
    </tbody>
</table>


## `BddVariableSet` (frozen)

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddVariableSet::new</code></td>
            <td rowspan="5"><code>BddVariableSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::new_anonymous</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::from_iterator[String]</code></td> 
        </tr>
        <tr>
            <td><code>BddVariableSet::from[Vec[String]]</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::from[Vec[&str]]</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.__getnewargs__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Variable introspection</td></tr>
        <tr>
            <td><code>BddVariableSet::num_vars</code></td>
            <td><code>BddVariableSet.variable_count</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::variables</code></td>
            <td><code>BddVariableSet.variable_ids</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.variable_names</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::var_by_name</code></td>
            <td><code>BddVariableSet.find_variable</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::name_of</code></td>
            <td><code>BddVariableSet.get_variable_name</code></td>
        </tr> 
        <tr><td colspan="2" align="center">BDD constructors</td></tr>
        <tr>
            <td><code>BddVariableSet::mk_false</code></td>
            <td><code>BddVariableSet.mk_false</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_true</code></td>
            <td><code>BddVariableSet.mk_true</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddVariableSet.mk_const</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_literal</code></td>
            <td rowspan="5"><code>BddVariableSet.mk_literal</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_var</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_var_by_name</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_not_var</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_not_var_by_name</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_conjunctive_clause</code></td>
            <td><code>BddVariableSet.mk_conjunctive_clause</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_disjunctive_clause</code></td>
            <td><code>BddVariableSet.mk_disjunctive_clause</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_cnf</code></td>
            <td><code>BddVariableSet.mk_cnf</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_dnf</code></td>
            <td><code>BddVariableSet.mk_dnf</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_sat_exactly_k</code></td>
            <td><code>BddVariableSet.mk_sat_exactly_k</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::mk_sat_up_to_k</code></td>
            <td><code>BddVariableSet.mk_sat_up_to_k</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::eval_expression</code></td>
            <td rowspan="3"><code>BddVariableSet.eval_expression</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::eval_expression_string</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::safe_eval_expression</code></td>
        </tr>
        <tr>
            <td><code>BddVariableSet::transfer_from</code></td>
            <td><code>BddVariableSet.transfer_from</code></td>
        </tr>
    </tbody>
</table>

## `BddValuation`

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddValuation::new</code></td>
            <td rowspan="3"><code>BddValuation.__init__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::all_false</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::try_from<&BddPartialValuation></code></td>
        </tr>
        <tr>
            <td><code>BddValuation::all_true</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BddValuation::eq</code></td>
            <td><code>BddValuation.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::cmp</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BddValuation::hash</code></td>
            <td><code>BddValuation.__hash__</code></td>
        </tr> 
        <tr>
            <td><code>BddValuation::to_string()</code></td>
            <td><code>BddValuation.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuation.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuation.__getnewargs__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuation.__ctx__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::num_vars</code></td>
            <td><code>BddValuation.__len__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::index</code></td>
            <td rowspan="2"><code>BddValuation.__getitem__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::value</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::set_value</code></td>
            <td rowspan="4"><code>BddValuation.__setitem__</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::clear</code></td> 
        </tr>
        <tr>
            <td><code>BddValuation::set</code></td> 
        </tr>
        <tr>
            <td><code>BddValuation::flip_value</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuation.__contains__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Iteration / conversion</td></tr>
        <tr>
            <td><code>BddValuation::vector</code></td>
            <td><code>BddValuation.values</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuation.keys</code></td>
        </tr>
        <tr>
            <td><code>BddValuation::to_values</code></td>
            <td><code>BddValuation.items</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>BddValuation::extends</code></td>
            <td><code>BddValuation.extends</code></td>
        </tr> 
    </tbody>
</table>

## `BddPartialValuation`

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BddPartialValuation::empty</code></td>
            <td rowspan="3"><code>BddPartialValuation.__init__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::from_values</code></td> 
        </tr>
        <tr>
            <td><code>BddPartialValuation::from<&BddValuation></code></td> 
        </tr>
        <tr>
            <td><code>BddPartialValuation::eq</code></td>
            <td><code>BddPartialValuation.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::hash</code></td>
            <td><code>BddPartialValuation.__hash__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::to_string</code></td>
            <td><code>BddPartialValuation.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPartialValuation.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPartialValuation.__getnewargs__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPartialValuation.__ctx__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::cardinality</code></td>
            <td><code>BddPartialValuation.__len__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::get_value</code></td>
            <td><code>BddPartialValuation.__getitem__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::set_value</code></td>
            <td><code>BddPartialValuation.__setitem__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::unset_value</code></td>
            <td><code>BddPartialValuation.__delitem__</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::has_value</code></td>
            <td><code>BddPartialValuation.__contains__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Iteration / conversion</td></tr>
        <tr>
            <td rowspan="1"><code>BddPartialValuation::to_values</code></td>
            <td><code>BddPartialValuation.items</code></td>
        </tr>
        <tr> 
            <td></td>
            <td><code>BddPartialValuation.keys</code></td>
        </tr>
        <tr> 
            <td></td>
            <td><code>BddPartialValuation.values</code></td>
        </tr>
        <tr> 
            <td></td>
            <td><code>BddPartialValuation.to_dict</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr> 
        <tr>
            <td><code>BddPartialValuation::is_empty</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::extends</code></td>
            <td><code>BddPartialValuation.extends</code></td>
        </tr>
        <tr>
            <td><code>BddPartialValuation::last_fixed_variable</code></td>
            <td></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddPartialValuation.support_set</code></td>
        </tr>
    </tbody>
</table>

## `Bdd` (frozen)

The `IntoBdd` trait and the `bdd!` macro trait are currently largely ignored. 
The `Bdd` constructor should be sufficient, as it supports a wide variety of types.

In the future, we can consider adding logical operations on BDDs as 
["infix operators"](https://code.activestate.com/recipes/384122/). But this 
poses a few challenges:
 
 * If we want to make this available for multiple types,
we will need either complicated generics, or heavy use of `Any`.
 * We still cannot override Python keywords like `and` and `or`, so we would have
   to use something else.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>Bdd::from<&BddValuation></code></td>
            <td rowspan="5"><code>Bdd.__init__</code></td>
        </tr>
        <tr>
            <td><code>Bdd::read_as_bytes</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::read_as_string</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::from_bytes</code></td>
        </tr>
        <tr>
            <td><code>Bdd::from_string</code></td>
        </tr>
        <tr>
            <td><code>Bdd::hash</code></td>
            <td><code>Bdd.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Bdd.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Bdd.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Bdd.__repr__</code></td>
        </tr> 
        <tr>
            <td></td>
            <td><code>Bdd.__getnewargs__</code></td>
        </tr> 
        <tr>
            <td></td>
            <td><code>Bdd.__ctx__</code></td>
        </tr> 
        <tr>
            <td><code>Bdd::eval_in</code></td>
            <td><code>Bdd.__call__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Conversions and serialization</td></tr>
        <tr>
            <td><code>Bdd::to_bytes</code></td>
            <td rowspan="2"><code>Bdd.data_bytes</code></td>
        </tr>
        <tr>
            <td><code>Bdd::write_as_bytes</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::to_string</code></td>
            <td rowspan="2"><code>Bdd.data_string</code></td>
        </tr>
        <tr>
            <td><code>Bdd::write_as_string</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::to_dot_string</code></td>
            <td rowspan="2"><code>Bdd.to_dot</code></td>
        </tr>
        <tr>
            <td><code>Bdd::write_as_dot_string</code></td>
        </tr>
        <tr>
            <td><code>Bdd::to_boolean_expression</code></td>
            <td><code>Bdd.to_expression</code></td>
        </tr>
        <tr>
            <td><code>Bdd::to_dnf</code></td>
            <td rowspan="2"><code>Bdd.to_dnf</code></td>
        </tr>
        <tr>
            <td><code>Bdd::(_)to_optimized_dnf</code></td>
        </tr>
        <tr>
            <td><code>Bdd::to_cnf</code></td>
            <td><code>Bdd.to_cnf</code></td>
        </tr>
        <tr><td colspan="2" align="center">Structural properties and basic introspection</td></tr>
        <tr>
            <td rowspan="2"><code>Bdd::size</code></td>
            <td><code>Bdd.__len__</code></td>
        </tr> 
        <tr>
            <td><code>Bdd.node_count</code></td>
        </tr>
        <tr>
            <td><code>Bdd::size_per_variable</code></td>
            <td><code>Bdd.node_count_per_variable</code></td>
        </tr>
        <tr>
            <td><code>Bdd::eq</code></td>
            <td><code>Bdd.structural_eq</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Bdd.semantic_eq</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Bdd.implies</code></td>
        </tr>
        <tr>
            <td><code>Bdd::root_pointer</code></td>
            <td><code>Bdd.root</code></td>
        </tr>
        <tr>
            <td><code>Bdd::high_link_of</code></td>
            <td rowspan="2"><code>Bdd.node_links</code></td>
        </tr>
        <tr>
            <td><code>Bdd::low_link_of</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::var_of</code></td>
            <td><code>Bdd.node_variable</code></td>
        </tr>
        <tr>
            <td><code>Bdd::num_vars</code></td>
            <td><code>Bdd.variable_count</code></td>
        </tr>
        <tr>
            <td><code>Bdd::support_set</code></td>
            <td><code>Bdd.support_set</code></td>
        </tr>
        <tr>
            <td><code>Bdd::is_false</code></td>
            <td><code>Bdd.is_false</code></td>
        </tr>
        <tr>
            <td><code>Bdd::is_true</code></td>
            <td><code>Bdd.is_true</code></td>
        </tr>
        <tr>
            <td><code>Bdd::is_clause</code></td>
            <td><code>Bdd.is_clause</code></td>
        </tr>
        <tr>
            <td><code>Bdd::is_valuation</code></td>
            <td><code>Bdd.is_valuation</code></td>
        </tr>
        <tr>
            <td><code>Bdd::cardinality</code></td>
            <td rowspan="2"><code>Bdd.cardinality</code></td>
        </tr>
        <tr>
            <td><code>Bdd::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>Bdd::exact_clause_cardinality</code></td>
            <td><code>Bdd.clause_cardinality</code></td>
        </tr>
        <tr><td colspan="2" align="center">Boolean operations</td></tr>
        <tr>
            <td><code>Bdd::not</code></td>
            <td><code>Bdd.l_not</code></td>
        </tr>
        <tr>
            <td><code>Bdd::and</code></td>
            <td><code>Bdd.l_and</code></td>
        </tr>
        <tr>
            <td><code>Bdd::or</code></td>
            <td><code>Bdd.l_or</code></td>
        </tr>
        <tr>
            <td><code>Bdd::and</code></td>
            <td><code>Bdd.l_and_not</code></td>
        </tr>
        <tr>
            <td><code>Bdd::imp</code></td>
            <td><code>Bdd.l_imp</code></td>
        </tr>
        <tr>
            <td><code>Bdd::iff</code></td>
            <td><code>Bdd.l_iff</code></td>
        </tr>
        <tr>
            <td><code>Bdd::xor</code></td>
            <td><code>Bdd.l_xor</code></td>
        </tr>        
        <tr>
            <td><code>Bdd::if_then_else</code></td>
            <td><code>Bdd.if_then_else</code></td>
        </tr>
        <tr>
            <td><code>Bdd::binary_op</code></td>
            <td rowspan="4"><code>Bdd.apply2</code></td>
        </tr>
        <tr>
            <td><code>Bdd::binary_op_with_limit</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::fused_binary_flip_op</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::fused_binary_flip_op_with_limit</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::ternary_op</code></td>
            <td rowspan="2"><code>Bdd.apply3</code></td>
        </tr>
        <tr>
            <td><code>Bdd::fused_ternary_flip_op</code></td>
        </tr>
        <tr>
            <td><code>Bdd::check_binary_op</code></td>
            <td rowspan="2"><code>Bdd.check2</code></td>
        </tr>
        <tr>
            <td><code>Bdd::check_fused_binary_flip_op</code></td> 
        </tr>
        <tr><td colspan="2" align="center">Relational operations</td></tr>
        <tr>
            <td><code>Bdd::pick</code></td>
            <td rowspan="2"><code>Bdd.r_pick</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_pick</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::pick_random</code></td>
            <td rowspan="2"><code>Bdd.r_pick_random</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_pick_random</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::exists</code></td>
            <td rowspan="2"><code>Bdd.r_exists</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_exists</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::for_all</code></td>
            <td rowspan="2"><code>Bdd.r_for_all</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_for_all</code></td> 
        </tr>
        <tr>
            <td><code>Bdd::restrict</code></td>
            <td rowspan="2"><code>Bdd.r_restrict</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_restrict</code></td>
        </tr>
        <tr>
            <td><code>Bdd::select</code></td>
            <td rowspan="2"><code>Bdd.r_select</code></td>
        </tr>
        <tr>
            <td><code>Bdd::var_select</code></td>
        </tr>
        <tr> 
            <td><code>Bdd::binary_op_nested</code></td>
            <td><code>Bdd.apply_nested</code></td>
        </tr>
        <tr> 
            <td><code>Bdd::binary_op_with_exists</code></td>
            <td><code>Bdd.apply_with_exists</code></td>
        </tr>
        <tr> 
            <td><code>Bdd::binary_op_with_for_all</code></td>
            <td><code>Bdd.apply_with_for_all</code></td>
        </tr> 
        <tr><td colspan="2" align="center">Valuations / Clauses</td></tr>
        <tr>
            <td><code>Bdd::sat_witness</code></td>
            <td><code>Bdd.witness</code></td>
        </tr>
        <tr>
            <td><code>Bdd::first_valuation</code></td>
            <td><code>Bdd.valuation_first</code></td>
        </tr>
        <tr>
            <td><code>Bdd::last_valuation</code></td>
            <td><code>Bdd.valuation_last</code></td>
        </tr>
        <tr>
            <td><code>Bdd::random_valuation</code></td>
            <td><code>Bdd.valuation_random</code></td>
        </tr>
        <tr>
            <td><code>Bdd::most_negative_valuation</code></td>
            <td><code>Bdd.valuation_most_negative</code></td>
        </tr>
        <tr>
            <td><code>Bdd::most_positive_valuation</code></td>
            <td><code>Bdd.valuation_most_positive</code></td>
        </tr>
        <tr>
            <td><code>Bdd::sat_valuations</code></td>
            <td><code>Bdd.valuation_iterator</code></td>
        </tr>
        <tr>
            <td><code>Bdd::first_clause</code></td>
            <td><code>Bdd.clause_first</code></td>
        </tr>
        <tr>
            <td><code>Bdd::last_clause</code></td>
            <td><code>Bdd.clause_last</code></td>
        </tr>
        <tr>
            <td><code>Bdd::random_clause</code></td>
            <td><code>Bdd.clause_random</code></td>
        </tr>
        <tr>
            <td><code>Bdd::most_fixed_clause</code></td>
            <td><code>Bdd.clause_most_fixed</code></td>
        </tr>
        <tr>
            <td><code>Bdd::most_free_clause</code></td>
            <td><code>Bdd.clause_most_free</code></td>
        </tr>
        <tr>
            <td><code>Bdd::necessary_clause</code></td>
            <td><code>Bdd.clause_necessary</code></td>
        </tr>
        <tr>
            <td><code>Bdd::sat_clauses</code></td>
            <td><code>Bdd.clause_iterator</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>Bdd::substitute</code></td>
            <td><code>Bdd.substitute</code></td>
        </tr>
        <tr>
            <td><code>Bdd::rename_variable</code></td>
            <td><code>Bdd.rename</code></td>
        </tr>
        <tr>
            <td><code>Bdd::set_num_vars</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Bdd::from_nodes</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Bdd::to_nodes</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

## `op_function` module

Currently, it does not really make sense to export the functions from this module 
into Python just to receive them back as references. Instead, we use a different 
approach:

`Bdd` methods that accept arbitrary "op function" in Rust can accept an arbitrary 
callable Python object. They use it to build an exhaustive lookup table. 
The original Rust method is then called with a function that is based on 
this lookup table. Since the lookup tables are small, the overhead is acceptable 
for any sufficiently large `Bdd`.

## `BddPathIterator` and `ValuationsOfClauseIterator`

We do not export the iterator types directly, because the API is quite low level and 
frankly kind of weird. Instead, we have two Python-only types: `BddClauseIterator` 
and `BddValuationIterator`. These just go through all relevant clauses/valuations of 
a single `Bdd` and have no other public API. If you still want to replicate the behaviour 
of the Rust iterators, you can always create a `Bdd` representing a single clause 
(or a `True` BDD) and iterate over that.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody> 
        <tr>
            <td></td>
            <td><code>BddClauseIterator.__init__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddClauseIterator.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddClauseIterator.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddClauseIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddClauseIterator.__next__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuationIterator.__init__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuationIterator.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuationIterator.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuationIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BddValuationIterator.__next__</code></td>
        </tr>
    </tbody>
</table>

## `BooleanExpression` (frozen)

Originally, `BooleanExpression` has almost no API and we rely on using Rust pattern matching
for most non-trivial manipulation. However, this does not really work in Python, hence we have
added a bunch of utility methods similar to how `FnUpdate` is expected to work.

<!--suppress XmlDeprecatedElement -->
<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BooleanExpression::try_from(&str)</code></td>
            <td><code>BooleanExpression.__init__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.__hash__</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::eq</code></td>
            <td><code>BooleanExpression.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::to_string</code></td>
            <td><code>BooleanExpression.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.__getnewargs__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.__root__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.__call__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern constructors</td></tr>
        <tr>
            <td><code>BooleanExpression::Const</code></td>
            <td><code>BooleanExpression.mk_const</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Variable</code></td>
            <td><code>BooleanExpression.mk_var</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Not</code></td>
            <td><code>BooleanExpression.mk_not</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::And</code></td>
            <td><code>BooleanExpression.mk_and</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Or</code></td>
            <td><code>BooleanExpression.mk_or</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Imp</code></td>
            <td><code>BooleanExpression.mk_imp</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Iff</code></td>
            <td><code>BooleanExpression.mk_iff</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Xor</code></td>
            <td><code>BooleanExpression.mk_xor</code></td>
        </tr>
        <tr>
            <td><code>BooleanExpression::Cond</code></td>
            <td><code>BooleanExpression.mk_cond</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.mk_conjunction</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.mk_disjunction</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern tests</td></tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_cond</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_literal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.is_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern destructors</td></tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_cond</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_literal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanExpression.as_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>BooleanExpression::support_set</code></td>
            <td><code>BooleanExpression.support_set</code></td>
        </tr>
    </tbody>
</table>
