# Biodivine `lib-bdd` API coverage

This document should be up-to-date with `lib-bdd` version `0.5.2`.

### `biodivine_std` module

This is mostly a legacy module that will not be used anymore once we
get rid of all the remaining use-cases. Hence, we do not include it 
in the public Python API.

The only major exception is the `Set` trait, which is still used by
the symbolic sets. In the future, we could replace this trait with 
a Python `Protocol`. However, the trait is not really used anywhere
in the public API right now, it mostly just simplifies/separates
implementation. Hence, we can omit it too.

### `Sign`, `Monotonicity`, and `ExtendedBoolean`

These enums are mostly just for additional type safety and are not really
needed in Python. In Python, we can replace them with a `Literal` type 
alias or optional types.

In particular, a `Sign` value translates to the following constant values:
- `positive`, or `+`;
- `negative`, or `-`;
- `None` if optional.

The `Monotonicity` enum is replaced with `Sign | None`.

Finally, an `ExtendedBoolean` is simply `bool | None`.

### `Variable`, `Parameter` and `Regulation`

Since these don't really have much use (`Variable` only contains the name,
`Parameter` name and arity), we don't have them in Python. Instead, you
can ask for a name/arity using a "context object".

For `Regulation`, we can instead use a `TypedDict` and we should be ok. 

## `VariableId` and `ParameterId` (frozen)

These work essentially the same way as the `BddVariable` class:

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
            <td><code>VariableId::from_index</code></td>
            <td rowspan="2"><code>VariableId.__init__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::from&lt;usize&gt;</code></td> 
        </tr>
        <tr>
            <td><code>VariableId::eq</code></td>
            <td rowspan="2"><code>VariableId.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::cmp</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>VariableId.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VariableId.__repr__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::hash</code></td>
            <td><code>VariableId.__hash__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::to_index</code></td>
            <td><code>VariableId.__index__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VariableId.__getnewargs__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::try_from_usize</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

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
            <td><code>ParameterId::from_index</code></td>
            <td rowspan="2"><code>ParameterId.__init__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::from&lt;usize&gt;</code></td> 
        </tr>
        <tr>
            <td><code>ParameterId::eq</code></td>
            <td rowspan="2"><code>ParameterId.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::cmp</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>ParameterId.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ParameterId.__repr__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::hash</code></td>
            <td><code>ParameterId.__hash__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::to_index</code></td>
            <td><code>ParameterId.__index__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ParameterId.__getnewargs__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::try_from_usize</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

### `RegulatoryGraph`

Currently, we don't export `RegulationIterator`. A copy of the data is
returned instead. We also do not provide the `Index` trait for `VariableId`
because there is no `Variable` object that could be returned by this 
operation, and just returning names seems confusing.

Finally, we also do not export `SdGraph`. Instead, the available algorithms
are exported as part of the `RegulatoryGraph`, because it is usually 
"sufficiently fast" at this point, and we don't pollute the API as much
as in Rust due to use of default argument values and polymorphism.

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
            <td><code>RegulatoryGraph::new</code></td>
            <td rowspan="2"><code>RegulatoryGraph.__init__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::try_from_string_regulations</code></td> 
        </tr> 
        <tr>
            <td><code>RegulatoryGraph::eq</code></td>
            <td><code>RegulatoryGraph.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.__getnewargs__</code></td>
        </tr>
        <tr>
            <td rowspan="2"><code>RegulatoryGraph::clone</code></td>
            <td><code>RegulatoryGraph.__copy__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph.__deepcopy__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Conversions</td></tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.from_file</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::try_from(&str)</code></td>
            <td><code>RegulatoryGraph.from_aeon</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.to_aeon</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::to_dot</code></td>
            <td><code>RegulatoryGraph.to_dot</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::write_as_dot</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Introspection and mutability</td></tr>
        <tr>
            <td><code>RegulatoryGraph::num_vars</code></td>
            <td><code>RegulatoryGraph.variable_count</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::variable_names</code></td>
            <td><code>RegulatoryGraph.variable_names</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::variables</code></td>
            <td><code>RegulatoryGraph.variables</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::get_variable</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::find_variable</code></td>
            <td><code>RegulatoryGraph.find_variable</code></td>
        </tr>        
        <tr>
            <td><code>RegulatoryGraph::get_variable_name</code></td>
            <td><code>RegulatoryGraph.get_variable_name</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::set_variable_name</code></td>
            <td><code>RegulatoryGraph.set_variable_name</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.regulation_count</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::regulations</code></td>
            <td><code>RegulatoryGraph.regulations</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.regulation_strings</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::find_regulation</code></td>
            <td><code>RegulatoryGraph.find_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::add_raw_regulation</code></td>
            <td rowspan="3"><code>RegulatoryGraph.add_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::add_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::add_string_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::remove_regulation</code></td>
            <td><code>RegulatoryGraph.remove_regulation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph.ensure_regulation</code></td>
        </tr>
        <tr><td colspan="2" align="center">Structural updates</td></tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::extend</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::drop</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::inline_variable</code></td>
        </tr>
        <tr><td colspan="2" align="center">Graph exploration and algorithms</td></tr>
        <tr>
            <td><code>RegulatoryGraph::regulators</code></td>
            <td><code>RegulatoryGraph.predecessors</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::targets</code></td>
            <td><code>RegulatoryGraph.successors</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::transitive_regulators</code></td>
            <td rowspan="3"><code>RegulatoryGraph.backward_reachable</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::backward_reachable</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_backward_reachable</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::transitive_targets</code></td>
            <td rowspan="3"><code>RegulatoryGraph.forward_reachable</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::forward_reachable</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_forward_reachable</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::feedback_vertex_set</code></td>
            <td rowspan="4"><code>RegulatoryGraph::feedback_vertex_set</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::parity_feedback_vertex_set</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_feedback_vertex_set</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_parity_feedback_vertex_set</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::independent_cycles</code></td>
            <td rowspan="4"><code>RegulatoryGraph::independent_cycles</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::independent_parity_cycles</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_independent_cycles</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_parity_independent_cycles</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::strongly_connected_components</code></td>
            <td rowspan="5"><code>RegulatoryGraph::strongly_connected_components
</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::restricted_strongly_connected_components
</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::components
</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_strongly_connected_components</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::strongly_connected_components</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::weakly_connected_components</code></td>
            <td rowspan="2"><code>RegulatoryGraph::weakly_connected_components
</code
></td>
        </tr>
        <tr>
            <td><code>SdGraph::restricted_weakly_connected_components</code></td> 
        </tr>
        <tr>
            <td><code>RegulatoryGraph::shortest_cycle</code></td>
            <td rowspan="4"><code>RegulatoryGraph::shortest_cycle</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::shortest_parity_cycle</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::shortest_cycle</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::shortest_parity_cycle</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::shortest_cycle_length</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SdGraph::shortest_parity_cycle_length</code></td>
<td></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>RegulatoryGraph::is_valid_name</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SdGraph::mk_all_vertices</code></td>
            <td></td>
        </tr> 
    </tbody>
</table>

### `BooleanNetwork`

A `BooleanNetwork` inherits from `RegulatoryGraph` and hence we need to
re-implement methods that mutate the graph to reflect both underlying objects.

As for `RegulatoryGraph`, we do not actually implement any `Index` traits, and
we export trivial iterators as lists.

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
            <td><code>BooleanNetwork::new</code></td>
            <td><code>BooleanNetwork.__init__</code></td>
        </tr> 
        <tr>
            <td><code>BooleanNetwork::eq</code></td>
            <td><code>RegulatoryGraph.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.__getnewargs__</code></td>
        </tr>
        <tr>
            <td rowspan="2"><code>BooleanNetwork::clone</code></td>
            <td><code>BooleanNetwork.__copy__</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork.__deepcopy__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Overriding or inheriting <code>RegulatoryGraph</code></td></tr>
        <tr>
            <td><code>BooleanNetwork::try_from_file</code></td>
            <td><code>BooleanNetwork.from_file</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from(&str)</code></td>
            <td><code>BooleanNetwork.from_aeon</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_string</code></td>
            <td><code>BooleanNetwork.to_aeon</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::num_vars</code></td>
            <td><code>RegulatoryGraph.variable_count</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::variables</code></td>
            <td><code>RegulatoryGraph.variables</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_variable_name</code></td>
            <td><code>RegulatoryGraph.get_variable_name</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::set_variable_name</code></td>
            <td><code>BooleanNetwork.set_variable_name</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.add_regulation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.remove_regulation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.ensure_regulation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.extend</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.drop</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::inline_variable</code></td>
            <td><code>BooleanNetwork.inline_variable</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::regulators</code></td>
            <td><code>RegulatoryGraph.predecessors</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::targets</code></td>
            <td><code>RegulatoryGraph.successors</code></td>
        </tr>
        <tr><td colspan="2" align="center">Conversions</td></tr>
        <tr>
            <td><code>BooleanNetwork::as_graph</code></td>
            <td><code>BooleanNetwork.as_graph</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::as_graph_mut</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_bnet</code></td>
            <td><code>BooleanNetwork.from_bnet</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_bnet</code></td>
            <td><code>BooleanNetwork.to_bnet</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_sbml</code></td>
            <td><code>BooleanNetwork.from_sbml</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_sbml_strict</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_sbml</code></td>
            <td><code>BooleanNetwork.to_sbml</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::write_as_sbml</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Introspection and mutability</td></tr>
        <tr>
            <td><code>BooleanNetwork::num_parameters</code></td>
            <td><code>BooleanNetwork.explicit_parameter_count</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::num_implicit_parameters</code></td>
            <td><code>BooleanNetwork.implicit_parameter_count</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::parameters</code></td>
            <td><code>BooleanNetwork.explicit_parameters</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::implicit_parameters</code></td>
            <td><code>BooleanNetwork.implicit_parameters</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.explicit_parameter_names</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.get_explicit_parameter_name</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork.get_explicit_parameter_arity</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::find_parameter</code></td>
            <td><code>BooleanNetwork.find_explicit_parameter</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_parameter</code></td>
            <td><code>BooleanNetwork.add_explicit_parameter</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_update_function</code></td>
            <td><code>BooleanNetwork.get_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_update_function</code></td>
            <td rowspan="3"><code>BooleanNetwork.set_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::set_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_string_update_function</code></td>
        </tr>
        <tr><td colspan="2" align="center">Structural updates</td></tr>
        <tr>
            <td><code>BooleanNetwork::infer_valid_graph</code></td>
            <td><code>BooleanNetwork.infer_valid_graph</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::inline_constants</code></td>
            <td><code>BooleanNetwork.inline_constants</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::inline_inputs</code></td>
            <td><code>BooleanNetwork.inline_inputs</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::prune_unused_parameters</code></td>
            <td><code>BooleanNetwork.prune_unused_parameters</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>BooleanNetwork::is_valid_name</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::get_variable</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::get_parameter</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::parameter_appears_in</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

### `FnUpdate` (aka `UpdateFunction`)

This largely follows the structure of `BooleanExpression` in `lib-bdd`, but of course
extended to cover uninterpreted functions. Also, an `UpdateFunction` carries a reference
to an underlying "context" in which it was created to resolve variable and function names.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Special methods</td></tr>
    </tbody>
</table>