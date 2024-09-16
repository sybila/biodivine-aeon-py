# Biodivine `lib-bdd` API coverage

This document should be up-to-date with `lib-bdd` version `0.5.10`.

### `biodivine_std` module

This is mostly a legacy module that will not be used anymore once we
get rid of all the remaining use-cases. Hence, we do not include it 
in the public Python API.

The only major exception is the `Set` trait, which is still used by
the symbolic sets. In the future, we could replace this trait with 
a Python `Protocol`. However, the trait is not really used anywhere
in the public API right now, it mostly just simplifies/separates
implementation. Hence, we can omit it too.

### `Sign`, `Monotonicity`, `ExtendedBoolean` and `BinaryOp`

These enums are mostly just for additional type safety and are not really
needed in Python. In Python, we can replace them with a `Literal` type 
alias or optional types.

In particular, a `Sign` value translates to the following constant values:
- `positive`, or `+`;
- `negative`, or `-`;
- `None` if optional.

The `Monotonicity` enum is replaced with `Sign | None`.

An `ExtendedBoolean` is simply `bool | None`.

Finally, a `BinaryOp` is `and`/`or`/`imp`/`iff`/`xor`.

### `Variable`, `Parameter`, `Regulation`, `Space`, and `SbmlLayout`

Since these don't really have much use (`Variable` only contains the name,
`Parameter` name and arity), we don't have them in Python. Instead, you
can ask for a name/arity using a "context object".

For `Regulation`, we can instead use a `TypedDict` and we should be ok.

For `Space`, we default to normal Python dictionaries. It's easier to 
include dynamic types in those.

For `SbmlLayout`, we currently do not provide any way to access this.
This will be later added once the SBML library matures.

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

## `RegulatoryGraph`

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
            <td><code>RegulatoryGraph::remove_regulation_constraints</code></td>
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
            <td><code>SdGraph::(_)restricted_feedback_vertex_set</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::(_)restricted_parity_feedback_vertex_set</code></td>
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
            <td><code>SdGraph::(_)restricted_independent_cycles</code></td>
        </tr>
        <tr>
            <td><code>SdGraph::(_)restricted_parity_independent_cycles</code></td>
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
            <td><code>SdGraph::(_)restricted_strongly_connected_components</code></td>
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
            <td><code>SdGraph::(_)restricted_weakly_connected_components</code></td> 
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

## `BooleanNetwork`

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
            <td><code>BooleanNetwork.to_graph</code></td>
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
            <td><code>BooleanNetwork::remove_static_constraints</code></td>
            <td><code>BooleanNetwork.remove_regulation_constraints</code></td>
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

## `FnUpdate` (aka `UpdateFunction`)

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
        <tr>
            <td><code>FnUpdate::try_from_str</code></td>
            <td rowspan="2"><code>UpdateFunction.__init__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::try_from_expression</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::hash</code></td>
            <td><code>UpdateFunction.__hash__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::eq</code></td>
            <td><code>UpdateFunction.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::to_string</code></td>
            <td><code>UpdateFunction.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.__getnewargs__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.__root__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.__ctx__</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::contains_variable</code></td>
            <td rowspan="2"><code>UpdateFunction.__contains__</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::contains_parameter</code></td> 
        </tr>
        <tr><td colspan="2" align="center">Pattern constructors</td></tr>
        <tr>
            <td><code>FnUpdate::Const</code></td>
            <td rowspan="3"><code>UpdateFunction.mk_const</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_false</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_true</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Var</code></td>
            <td rowspan="2"><code>UpdateFunction.mk_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Param</code></td>
            <td rowspan="3"><code>UpdateFunction.mk_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_basic_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Not</code></td>
            <td rowspan="2"><code>UpdateFunction.mk_not</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_not</code></td>
        </tr>
        <tr>
            <td rowspan="5"><code>FnUpdate::Binary</code></td>
            <td><code>UpdateFunction.mk_and</code></td>
        </tr>
        <tr> 
            <td><code>UpdateFunction.mk_or</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.mk_imp</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.mk_iff</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.mk_xor</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_binary</code></td>
            <td><code>UpdateFunction.mk_binary</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_conjunction</code></td>
            <td><code>UpdateFunction.mk_conjunction</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_disjunction</code></td>
            <td><code>UpdateFunction.mk_disjunction</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern tests</td></tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_param</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_literal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.is_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Method constructors</td></tr>
        <tr>
            <td><code>FnUpdate::negation</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::and</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::or</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::implies</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::iff</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::xor</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern destructors</td></tr>
        <tr>
            <td><code>FnUpdate::as_const</code></td>
            <td><code>UpdateFunction.as_const</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_var</code></td>
            <td><code>UpdateFunction.as_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_param</code></td>
            <td><code>UpdateFunction.as_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_not</code></td>
            <td><code>UpdateFunction.as_not</code></td>
        </tr>
        <tr>
            <td rowspan="5"><code>FnUpdate::as_binary</code></td>
            <td><code>UpdateFunction.as_and</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.as_or</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.as_imp</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.as_iff</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction.as_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction.as_literal</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_binary</code></td>
            <td><code>UpdateFunction.as_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>FnUpdate::collect_arguments</code></td>
            <td><code>UpdateFunction.support_variables</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::collect_parameters</code></td>
            <td><code>UpdateFunction.support_parameters</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::substitute_variable</code></td>
            <td><code>UpdateFunction.substitute_all</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::rename_all</code></td>
            <td><code>UpdateFunction.rename_all</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::simplify_constants</code></td>
            <td><code>UpdateFunction.simplify_constants</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::distribute_negation</code></td>
            <td><code>UpdateFunction.distribute_negation</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::to_and_or_normal_form</code></td>
            <td><code>UpdateFunction.to_and_or_normal_form</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::build_from_bdd</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::evaluate</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::walk_postorder</code></td>
            <td></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::as_expression</code></td>
        </tr>
    </tbody>
</table>

## `ModelAnnotation`

Annotations are implemented essentially as special dictionaries, hence most of the functionality
is actually in the special methods. Also, the main `ModelAnnotation` is `frozen`, but just because 
it is actually referencing a hidden `_ModelAnnotation` type that has interior mutability and
actually holds a reference to the native map.

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
            <td><code>ModelAnnotation::new</code></td>
            <td rowspan="2"><code>ModelAnnotation.__init__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::with_value</code></td> 
        </tr>
        <tr>
            <td><code>ModelAnnotation::eq</code></td>
            <td><code>ModelAnnotation.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.__copy__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::clone</code></td>
            <td><code>ModelAnnotation.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::to_string</code></td>
            <td><code>ModelAnnotation.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.__len__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::get_child</code></td>
            <td><code>ModelAnnotation.__getitem__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::get_mut_child</code></td>
            <td rowspan="2"><code>ModelAnnotation.__setitem__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::ensure_child</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.__delitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.__contains__</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::value</code></td>
            <td rowspan="2"><code>ModelAnnotation.get_value</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::get_value</code></td> 
        </tr>
        <tr>
            <td><code>ModelAnnotation::value_mut</code></td>
            <td rowspan="3"><code>ModelAnnotation.set_value</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::ensure_value</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::append_value</code></td> 
        </tr>
        <tr><td colspan="2" align="center">Conversions</td></tr>
        <tr>
            <td><code>ModelAnnotation::from_model_string</code></td>
            <td><code>ModelAnnotation.from_aeon</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ModelAnnotation.from_file</code></td>
        </tr>
        <tr>
            <td rowspan="3"><code>ModelAnnotation::children</code></td>
            <td><code>ModelAnnotation.values</code></td>
        </tr>
        <tr> 
            <td><code>ModelAnnotation.keys</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation.items</code></td>
        </tr>
        <tr>
            <td><code>ModelAnnotation::children_mut</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

## `SymbolicContext` (`frozen`)

For now, the decoding methods are left not implemented, as their current names are rather confusing. We should
design a nicer API for this down the line.

Right now, `FunctionTable` is not exported. Instead, we just convert it to a `list`.

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
            <td><code>SymbolicContext::new</code></td>
            <td><code>SymbolicContext.__init__</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::with_extra_state_variables</code></td> 
        </tr>
        <tr>
            <td><code>SymbolicContext::eq</code></td>
            <td><code>SymbolicContext.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.__str__</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::clone</code></td>
            <td><code>SymbolicContext.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.__deepcopy__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Symbolic variable mapping</td></tr>
        <tr>
            <td><code>SymbolicContext::num_state_variables</code></td>
            <td><code>SymbolicContext.network_variable_count</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.network_variable_names</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::network_variables</code></td>
            <td><code>SymbolicContext.network_variables</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::state_variables</code></td>
            <td><code>SymbolicContext.network_bdd_variables</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::find_network_variable</code></td>
            <td><code>SymbolicContext.find_network_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::find_state_variable</code></td>
            <td rowspan="2"><code>SymbolicContext.find_network_bdd_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_state_variable</code></td> 
        </tr>
        <tr>
            <td><code>SymbolicContext::get_network_variable_name</code></td>
            <td><code>SymbolicContext.get_network_variable_name</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::num_extra_state_variables</code></td>
            <td><code>SymbolicContext.extra_bdd_variable_count</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::all_extra_state_variables</code></td>
            <td><code>SymbolicContext.extra_bdd_variables_list</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::extra_state_variables</code></td>
            <td rowspan="3"><code>SymbolicContext.extra_bdd_variables</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::extra_state_variables_by_offset</code></td> 
        </tr>
        <tr>
            <td><code>SymbolicContext::get_extra_state_variable</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.explicit_function_count</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::network_parameters</code></td>
            <td><code>SymbolicContext.explicit_functions</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.explicit_function_bdd_variables_list</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.explicit_functions_bdd_variables</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.implicit_functions_count</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::network_implicit_parameters</code></td>
            <td><code>SymbolicContext.implicit_functions</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.implicit_functions_bdd_variables_list</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.implicit_functions_bdd_variables</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.function_count</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.functions</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::parameter_variables</code></td>
            <td><code>SymbolicContext.functions_bdd_variables_list</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicContext.functions_bdd_variables</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::find_network_parameter</code></td>
            <td><code>SymbolicContext.find_function</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_network_parameter_name</code></td>
            <td><code>SymbolicContext.get_function_name</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_network_parameter_arity</code></td>
            <td rowspan="2"><code>SymbolicContext.get_function_arity</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_network_implicit_parameter_arity</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_explicit_function_table</code></td>
            <td rowspan="2"><code>SymbolicContext.get_function_table</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::get_implicit_function_table</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::num_parameter_variables</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Encoding methods</td></tr>
        <tr>
            <td><code>SymbolicContext::mk_constant</code></td>
            <td><code>SymbolicContext.mk_constant</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_state_variable_is_true</code></td>
            <td><code>SymbolicContext.mk_network_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_constant</code></td>
            <td><code>SymbolicContext.mk_extra_bdd_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_uninterpreted_function_is_true</code></td>
            <td rowspan="2"><code>SymbolicContext.mk_function</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_implicit_function_is_true</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_fn_update_true</code></td>
            <td><code>SymbolicContext.mk_update_function</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_function_table_true</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Decoding methods</td></tr>
        <tr>
            <td><code>SymbolicContext::instantiate_implicit_function</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::instantiate_uninterpreted_function</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::instantiate_fn_update</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::mk_instantiated_fn_update</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>SymbolicContext::bdd_variable_set</code></td>
            <td><code>SymbolicContext.bdd_variable_set</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::transfer_from</code></td>
            <td><code>SymbolicContext.transfer_from</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::to_canonical_context</code></td>
            <td><code>SymbolicContext.as_canonical_context</code></td>
        </tr>
        <tr>
            <td><code>SymbolicContext::eliminate_network_variable</code></td>
            <td><code>SymbolicContext.eliminate_network_variable</code></td>
        </tr>
    </tbody>
</table>

## `SymbolicSpaceContext` (`frozen`)

Inherits from `SymbolicContext`. Extra symbolic variables are still accessible
using the normal methods, but new methods are added to facilitate the mapping.

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
            <td><code>SymbolicSpaceContext::new</code></td>
            <td><code>SymbolicSpaceContext.__init__</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::eq</code></td>
            <td><code>SymbolicSpaceContext.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicSpaceContext.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicSpaceContext.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SymbolicSpaceContext.__deepcopy__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Symbolic variable mapping</td></tr>
        <tr>
            <td><code>SymbolicSpaceContext::get_negative_variable</code></td>
            <td><code>SymbolicSpaceContext.get_negative_space_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::get_positive_variable</code></td>
            <td><code>SymbolicSpaceContext.get_positive_space_variable</code></td>
        </tr>
        <tr><td colspan="2" align="center">Encoding methods</td></tr>
        <tr>
            <td><code>SymbolicSpaceContext::(_)mk_can_go_to_true</code></td>
            <td><code>SymbolicSpaceContext.mk_can_go_to_true</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_empty_colored_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_empty_colored_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_empty_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_empty_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_unit_colored_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_unit_colored_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_unit_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_unit_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_unit_bdd</code></td>
            <td><code>SymbolicSpaceContext.mk_unit_bdd</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::(_)mk_sub_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_sub_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::(_)mk_super_spaces</code></td>
            <td><code>SymbolicSpaceContext.mk_super_spaces</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::mk_space</code></td>
            <td><code>SymbolicSpaceContext.mk_singleton</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::spaces_to_vertices</code></td>
            <td><code>SymbolicSpaceContext.spaces_to_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::vertices_to_spaces</code></td>
            <td><code>SymbolicSpaceContext.vertices_to_spaces</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>SymbolicSpaceContext::bdd_variable_set</code></td>
            <td><i>inherited</i></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::get_state_variable</code></td>
            <td><i>inherited</i></td>
        </tr>
        <tr>
            <td><code>SymbolicSpaceContext::inner_context</code></td>
            <td><i>automatically through inheritence</i></td>
        </tr>
    </tbody>
</table>

## `ColorSet`, `VertexSet`, `SpaceSet`, `ColoredVertexSet`, and `ColoredSpaceSet` (`frozen`)

Currently, symbolic sets hold a reference to the underlying `SymbolicContext`. As such, they
cannot be serialized easily. For now, you should use BDD serialization instead.

`IterableVertices` are not exported because they are deprecated. For more info on iterators, 
see below.

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
            <td><code>GraphColors::new</code></td>
            <td><code>ColorSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::eq</code></td>
            <td><code>ColorSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorSet.__str__</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::clone</code></td>
            <td><code>ColorSet.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>GraphColors.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::hash</code></td>
            <td><code>ColorSet.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorSet.__ctx__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Set operations</td></tr>
        <tr>
            <td><code>GraphColors::approx_cardinality</code></td>
            <td rowspan="2"><code>ColorSet.cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::intersect</code></td>
            <td><code>ColorSet.intersect</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::minus</code></td>
            <td><code>ColorSet.minus</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::union</code></td>
            <td><code>ColorSet.union</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::is_empty</code></td>
            <td><code>ColorSet.is_empty</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::is_subset</code></td>
            <td><code>ColorSet.is_subset</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::is_singleton</code></td>
            <td><code>ColorSet.is_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::is_subspace</code></td>
            <td><code>ColorSet.is_subspace</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::pick_singleton</code></td>
            <td><code>ColorSet.pick_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::symbolic_size</code></td>
            <td><code>ColorSet.symbolic_size</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorSet.extend_with_vertices</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorSet.extend_with_spaces</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>GraphColors::as_bdd</code></td>
            <td rowspan="2"><code>ColorSet.to_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::into_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphColors::copy</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColors::fn_update_projection</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColors::raw_projection</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColors::to_dot_string</code></td>
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
            <td><code>GraphVertices::new</code></td>
            <td><code>VertexSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::eq</code></td>
            <td><code>VertexSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexSet.__str__</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::clone</code></td>
            <td><code>VertexSet.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexSet.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::hash</code></td>
            <td><code>VertexSet.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexSet.__ctx__</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::iter</code></td>
            <td rowspan="3"><code>VertexSet.__iter__</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::into_iter</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::materialize</code></td>
        </tr>
        <tr><td colspan="2" align="center">Set operations</td></tr>
        <tr>
            <td><code>GraphVertices::approx_cardinality</code></td>
            <td rowspan="2"><code>VertexSet.cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::intersect</code></td>
            <td><code>VertexSet.intersect</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::minus</code></td>
            <td><code>VertexSet.minus</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::union</code></td>
            <td><code>VertexSet.union</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::is_empty</code></td>
            <td><code>VertexSet.is_empty</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::is_subset</code></td>
            <td><code>VertexSet.is_subset</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::is_singleton</code></td>
            <td><code>VertexSet.is_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::is_subspace</code></td>
            <td><code>VertexSet.is_subspace</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::pick_singleton</code></td>
            <td><code>VertexSet.pick_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::symbolic_size</code></td>
            <td><code>VertexSet.symbolic_size</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::to_singleton_spaces</code></td>
            <td><code>VertexSet.to_singleton_spaces</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexSet.extend_with_colors</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>GraphVertices::as_bdd</code></td>
            <td rowspan="2"><code>VertexSet.to_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::into_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::state_projection</code></td>
            <td rowspan="2"><code>VertexSet.items</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::raw_projection</code></td>
        </tr>
        <tr>
            <td><code>GraphVertices::copy</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphVertices::to_dot_string</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphVertices::fix_network_variable</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphVertices::restrict_network_variable</code></td>
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
            <td><code>NetworkSpaces::new</code></td>
            <td><code>SpaceSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::eq</code></td>
            <td><code>SpaceSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceSet.__str__</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::clone</code></td>
            <td><code>SpaceSet.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceSet.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::hash</code></td>
            <td><code>SpaceSet.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceSet.__ctx__</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::iter</code></td>
            <td rowspan="2"><code>SpaceSet.__iter__</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::into_iter</code></td>
        </tr>
        <tr><td colspan="2" align="center">Set operations</td></tr>
        <tr>
            <td><code>NetworkSpaces::approx_cardinality</code></td>
            <td rowspan="2"><code>SpaceSet.cardinality</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::intersect</code></td>
            <td><code>SpaceSet.intersect</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::minus</code></td>
            <td><code>SpaceSet.minus</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::union</code></td>
            <td><code>SpaceSet.union</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::is_empty</code></td>
            <td><code>SpaceSet.is_empty</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::is_subset</code></td>
            <td><code>SpaceSet.is_subset</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::is_singleton</code></td>
            <td><code>SpaceSet.is_singleton</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::pick_singleton</code></td>
            <td><code>SpaceSet.pick_singleton</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::symbolic_size</code></td>
            <td><code>SpaceSet.symbolic_size</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::to_vertices</code></td>
            <td><code>SpaceSet.to_vertices</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceSet.extend_with_colors</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>NetworkSpaces::as_bdd</code></td>
            <td rowspan="2"><code>SpaceSet.to_bdd</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::into_bdd</code></td>
        </tr>
        <tr>
            <td><code>NetworkSpaces::raw_projection</code></td>
            <td><code>SpaceSet.items</code></td>
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
            <td><code>GraphColoredVertices::new</code></td>
            <td><code>ColoredVertexSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::eq</code></td>
            <td><code>ColoredVertexSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredVertexSet.__str__</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::clone</code></td>
            <td><code>ColoredVertexSet.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredVertexSet.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::hash</code></td>
            <td><code>ColoredVertexSet.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredVertexSet.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredVertexSet.__ctx__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Set operations</td></tr>
        <tr>
            <td><code>GraphColoredVertices::approx_cardinality</code></td>
            <td rowspan="2"><code>ColoredVertexSet.cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::intersect</code></td>
            <td><code>ColoredVertexSet.intersect</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::minus</code></td>
            <td><code>ColoredVertexSet.minus</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::union</code></td>
            <td><code>ColoredVertexSet.union</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::is_empty</code></td>
            <td><code>ColoredVertexSet.is_empty</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::is_subset</code></td>
            <td><code>ColoredVertexSet.is_subset</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::is_singleton</code></td>
            <td><code>ColoredVertexSet.is_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::is_subspace</code></td>
            <td><code>ColoredVertexSet.is_subspace</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::symbolic_size</code></td>
            <td><code>ColoredVertexSet.symbolic_size</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::colors</code></td>
            <td><code>ColoredVertexSet.colors</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::vertices</code></td>
            <td><code>ColoredVertexSet.vertices</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::intersect_colors</code></td>
            <td><code>ColoredVertexSet.intersect_colors</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::intersect_vertices</code></td>
            <td><code>ColoredVertexSet.intersect_vertices</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::minus_colors</code></td>
            <td><code>ColoredVertexSet.minus_colors</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::minus_vertices</code></td>
            <td><code>ColoredVertexSet.minus_vertices</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::pick_color</code></td>
            <td><code>ColoredVertexSet.pick_color</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::pick_vertex</code></td>
            <td><code>ColoredVertexSet.pick_vertex</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::pick_singleton</code></td>
            <td><code>ColoredVertexSet.pick_singleton</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::to_singleton_spaces</code></td>
            <td><code>ColoredVertexSet.to_singleton_spaces</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>GraphColoredVertices::as_bdd</code></td>
            <td rowspan="2"><code>ColoredVertexSet.to_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::into_bdd</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::copy</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::fn_update_projection</code></td>
            <td rowspan="4"><code>ColoredVertexSet.items</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::state_projection</code></td> 
        </tr>
        <tr>
            <td><code>GraphColoredVertices::raw_projection</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::mixed_projection</code></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::to_dot_string</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::fix_network_variable</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>GraphColoredVertices::restrict_network_variable</code></td>
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
            <td><code>NetworkColoredSpaces::new</code></td>
            <td><code>ColoredSpaceSet.__init__</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::eq</code></td>
            <td><code>ColoredSpaceSet.__richcmp__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredSpaceSet.__str__</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::clone</code></td>
            <td><code>ColoredSpaceSet.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredSpaceSet.__deepcopy__</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::hash</code></td>
            <td><code>ColoredSpaceSet.__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredSpaceSet.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColoredSpaceSet.__ctx__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Set operations</td></tr>
        <tr>
            <td><code>NetworkColoredSpaces::approx_cardinality</code></td>
            <td rowspan="2"><code>ColoredSpaceSet.cardinality</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::exact_cardinality</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::intersect</code></td>
            <td><code>ColoredSpaceSet.intersect</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::minus</code></td>
            <td><code>ColoredSpaceSet.minus</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::union</code></td>
            <td><code>ColoredSpaceSet.union</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::is_empty</code></td>
            <td><code>ColoredSpaceSet.is_empty</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::is_subset</code></td>
            <td><code>ColoredSpaceSet.is_subset</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::is_singleton</code></td>
            <td><code>ColoredSpaceSet.is_singleton</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::symbolic_size</code></td>
            <td><code>ColoredSpaceSet.symbolic_size</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::colors</code></td>
            <td><code>ColoredSpaceSet.colors</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::spaces</code></td>
            <td><code>ColoredSpaceSet.spaces</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::intersect_colors</code></td>
            <td><code>ColoredSpaceSet.intersect_colors</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::intersect_spaces</code></td>
            <td><code>ColoredSpaceSet.intersect_spaces</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::minus_colors</code></td>
            <td><code>ColoredSpaceSet.minus_colors</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::minus_spaces</code></td>
            <td><code>ColoredSpaceSet.minus_spaces</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::pick_color</code></td>
            <td><code>ColoredSpaceSet.pick_color</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::pick_space</code></td>
            <td><code>ColoredSpaceSet.pick_space</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::pick_singleton</code></td>
            <td><code>ColoredSpaceSet.pick_singleton</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::to_colored_vertices</code></td>
            <td><code>ColoredSpaceSet.to_colored_vertices</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>NetworkColoredSpaces::as_bdd</code></td>
            <td rowspan="2"><code>ColoredSpaceSet.to_bdd</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::into_bdd</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::copy</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::fn_update_projection</code></td>
            <td rowspan="2"><code>ColoredSpaceSet.items</code></td>
        </tr>
        <tr>
            <td><code>NetworkColoredSpaces::raw_projection</code></td>
        </tr>
    </tbody>
</table>


## Symbolic iterators

These classes replace the functionality of various symbolic iterators 
(`GraphVertexIterator`, `FunctionTableIterator`, `OwnedRawSymbolicIterator`,
`MixedProjectionIterator`, `RawSymbolicIterator`, `FnUpdateProjectionIterator`,
`StateProjectionIterator`, `SymbolicIterator`, `SpaceIterator`) and projections (`MixedProjection`,
`RawProjection`, `StateProjection`, `FnUpdateProjection`). They are simplified significantly
compared to Rust because they support both normal and projected iteration simultaneously.
The API is designed based on similar "model classes" that are provided e.g. by Z3.

The new Python classes are designed to be instantiated exclusively from Rust and have only
minimal API necessary to retrieve the relevant values. All model classes are `frozen`.

### `_VertexModelIterator` and `VertexModel`

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
            <td></td>
            <td><code>_VertexModelIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>_VertexModelIterator.__next__</code></td>
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
            <td></td>
            <td><code>VertexModel.__ctx__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.__getitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.__contains__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.keys</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.values</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.items</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.to_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.to_named_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.to_valuation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VertexModel.to_symbolic</code></td>
        </tr>
    </tbody>
</table>

### `_ColorModelIterator` and `ColorModel`

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
            <td></td>
            <td><code>_ColorModelIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>_ColorModelIterator.__next__</code></td>
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
            <td></td>
            <td><code>ColorModel.__ctx__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.__getitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.__contains__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.keys</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.values</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.items</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.to_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.to_named_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.to_valuation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.to_symbolic</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.instantiate</code></td>
        </tr>
    </tbody>
</table>

### `_SpaceModelIterator` and `SpaceModel`

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
            <td></td>
            <td><code>_SpaceModelIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>_SpaceModelIterator.__next__</code></td>
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
            <td></td>
            <td><code>SpaceModel.__ctx__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.__getitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.__contains__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.keys</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.values</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.items</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.to_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ColorModel.to_named_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.to_valuation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>SpaceModel.to_symbolic</code></td>
        </tr>
    </tbody>
</table>


#### `_ColorVertexModelIterator` and `_ColorSpaceModelIterator`

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
            <td></td>
            <td><code>_ColorVertexModelIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>_ColorVertexModelIterator.__next__</code></td>
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
            <td></td>
            <td><code>_ColorSpaceModelIterator.__iter__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>_ColorSpaceModelIterator.__next__</code></td>
        </tr>
    </tbody>
</table>

## `AsynchronousGraph` (`frozen`)


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
            <td><code>SymbolicAsyncGraph::new</code></td>
            <td rowspan="4"><code>AsynchronousGraph.__init__</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::new_raw</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::with_custom_context</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::with_space_context</code></td> 
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.__str__</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::clone</code></td>
            <td><code>AsynchronousGraph.__copy__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.__deepcopy__</code></td>
        </tr>
        <tr><td colspan="2" align="center">Conversions</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::as_network</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::symbolic_context</code></td>
            <td><code>AsynchronousGraph.symbolic_context</code></td>
        </tr>
        <tr><td colspan="2" align="center">Network introspection</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::num_vars</code></td>
            <td><code>AsynchronousGraph.network_variable_count</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.network_variable_names</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::variables</code></td>
            <td><code>AsynchronousGraph.network_variables</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.find_network_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::get_variable_name</code></td>
            <td><code>AsynchronousGraph.get_network_variable_name</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::reconstruct_network</code></td>
            <td><code>AsynchronousGraph.reconstruct_network</code></td>
        </tr>
        <tr><td colspan="2" align="center">Symbolic constructors</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::empty_colored_vertices</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_empty_colored_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_empty_colored_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::empty_colors</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_empty_colors</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_empty_colors</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::empty_vertices</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_empty_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_empty_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::unit_colored_vertices</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_unit_colored_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_unit_colored_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::unit_colors</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_unit_colors</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_unit_colors</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::unit_vertices</code></td>
            <td rowspan="2"><code>AsynchronousGraph.mk_unit_vertices</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_unit_vertices</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.mk_function_colors</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>AsynchronousGraph.mk_function_row_colors</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::transfer_colors_from</code></td>
            <td rowspan="3"><code>AsynchronousGraph.transfer_from</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::transfer_vertices_from</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::transfer_from</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::vertex</code></td>
            <td rowspan="5">
                <code>AsynchronousGraph.mk_subspace</code>
                <br>
                <code>AsynchronousGraph.mk_subspace_vertices</code>
            </td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::fix_network_variable</code></td> 
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::fix_vertices_with_network_variable</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_partial_vertex</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_subspace</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::get_symbolic_fn_update</code></td>
            <td><code>AsynchronousGraph.mk_update_function</code></td>
        </tr>
        <tr><td colspan="2" align="center">Atomic pre- and post- operations</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::post</code></td>
            <td><code>AsynchronousGraph.post</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::will_post_out</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::will_post_within</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::pre</code></td>
            <td><code>AsynchronousGraph.pre</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::will_pre_out</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::will_pre_within</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_post</code></td>
            <td><code>AsynchronousGraph.var_post</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_post_out</code></td>
            <td><code>AsynchronousGraph.var_post_out</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_post_within</code></td>
            <td><code>AsynchronousGraph.var_post_within</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_pre</code></td>
            <td><code>AsynchronousGraph.var_pre</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_pre_out</code></td>
            <td><code>AsynchronousGraph.var_pre_out</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_pre_within</code></td>
            <td><code>AsynchronousGraph.var_pre_within</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_post</code></td>
            <td><code>AsynchronousGraph.can_post</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_post_out</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_post_within</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_pre</code></td>
            <td><code>AsynchronousGraph.can_pre</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_pre_out</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::can_pre_within</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_post</code></td>
            <td><code>AsynchronousGraph.var_can_post</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_post_out</code></td>
            <td><code>AsynchronousGraph.var_can_post_out</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_post_within</code></td>
            <td><code>AsynchronousGraph.var_can_post_within</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_pre</code></td>
            <td><code>AsynchronousGraph.var_can_pre</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_pre_out</code></td>
            <td><code>AsynchronousGraph.var_can_pre_out</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::var_can_pre_within</code></td>
            <td><code>AsynchronousGraph.var_can_pre_within</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::trap_forward</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::trap_backward</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::reach_forward</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::reach_backward</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::space_has_var_true</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::space_has_var_false</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::percolate_space</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::mk_subnetwork_colors</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::wrap_in_symbolic_subspace</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::wrap_in_subspace</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::universal_extra_variable_projection</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::existential_extra_variable_projection</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::is_trap_set</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::inline_symbolic</code></td>
            <td><code>AsynchronousGraph::inline_variable_symbolic</code></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::restrict</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::restrict_variable_in_graph</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>SymbolicAsyncGraph::pick_witness</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

## Algorithm classes


<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Algorithm methods</td></tr>
        <tr>
            <td><code>TrapSpaces::(_)minimize</code></td>
            <td><code>TrapSpaces.minimize</code></td>
        </tr>
        <tr>
            <td><code>TrapSpaces::(_)maximize</code></td>
            <td><code>TrapSpaces.maximize</code></td>
        </tr>
        <tr>
            <td><code>TrapSpaces::(_)essential_symbolic</code></td>
            <td><code>TrapSpaces.essential_symbolic</code></td>
        </tr>
        <tr>
            <td><code>TrapSpaces::(_)minimal_symbolic</code></td>
            <td><code>TrapSpaces.minimal_symbolic</code></td>
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
        <tr><td colspan="2" align="center">Algorithm methods</td></tr>
        <tr>
            <td><code>FixedPoints::(_)symbolic</code></td>
            <td><code>FixedPoints.symbolic</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::(_)symbolic_vertices</code></td>
            <td><code>FixedPoints.symbolic_vertices</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::(_)symbolic_colors</code></td>
            <td><code>FixedPoints.symbolic_colors</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::naive_symbolic</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_iterator</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::(_)symbolic_merge</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_projection</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::naive_symbolic</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::solver_iterator</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::solver_vertex_iterator</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::colver_color_iterator</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::make_fixed_points_solver</code></td>
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
        <tr><td colspan="2" align="center">Algorithm methods</td></tr>
        <tr>
            <td><code>SymbolicAsyncGraph::percolate_space</code></td>
            <td><code>Percolation.percolate_subspace</code></td>
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
        <tr><td colspan="2" align="center">Algorithm methods</td></tr>
        <tr>
            <td><code>Reachability::reach_fwd</code></td>
            <td><code>Reachability.reach_fwd</code></td>
        </tr>
        <tr>
            <td><code>Reachability::reach_bwd</code></td>
            <td><code>Reachability.reach_bwd</code></td>
        </tr>
        <tr>
            <td><code>Reachability::(_)reach</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Reachability::(_)reach_basic_saturation</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Reachability::reach_fwd_basic</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Reachability::reach_bwd_basic</code></td>
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
        <tr><td colspan="2" align="center">Algorithm methods</td></tr>
        <tr>
            <td><code>RegulationConstraint::mk_activation</code></td>
            <td><code>RegulationConstraint.mk_activation</code></td>
        </tr>
        <tr>
            <td><code>RegulationConstraint::mk_inhibition</code></td>
            <td><code>RegulationConstraint.mk_inhibition</code></td>
        </tr>
        <tr>
            <td><code>RegulationConstraint::mk_observable</code></td>
            <td><code>RegulationConstraint.mk_essential</code></td>
        </tr>
        <tr>
            <td><code>RegulationConstraint::infer_sufficient_regulation</code></td>
            <td><code>RegulationConstraint.infer_sufficient_regulation</code></td>
        </tr>
        <tr>
            <td><code>Reachability::fix_regulation</code></td>
            <td><i>Requires some new code to create regulation objects.</i></td>
        </tr>
    </tbody>
</table>