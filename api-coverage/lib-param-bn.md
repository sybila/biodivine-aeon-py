# Biodivine `lib-param-bn`

The list of relationships between public APIs in Rust and Python. "Trivial" items that do not have direct Python counterparts, like blanket trait implementations (`Into`, `From`, ...) or the `Debug` trait are intentionally omitted. For more information about individual Python functions, see the Python API documentation generated from the `biodivine_aeon.pyi` stub file.

### `async_graph`, `bdd_params` and `biodivine_std`

These are legacy modules which we no longer support and as such, they are not
part of the Python API.

### `Sign`, `Monotonicity`, and `ExtendedBoolean`

These enums are mostly just for additional type safety and are not really 
needed in Python (a string with predefined values or an optional bool should be 
enough).

In particular, a `Sign` value translates to the following constant values:
 - `positive`, or `+`;
 - `negative`, or `-`;
 - `None` if optional.

The `Monotonicity` value is then one of `activation`, `inhibition`, or `None`.

Finally, an `ExtendedBoolean` is simply `bool | None`.

### `Space`, `State`, and `UpdateFunction`

These cover the basic data objects that are part of the BN semantics. 
Note that these are not completely aligned with their Rust counterparts, but 
this design seems to make more sense (at least in retrospect). 

In terms of the symbolic encoding, these can all be constructed from either
a `BddValuation` or a suitable `BddPartialValuation` (assuming necessary 
variables are present). However, `UpdateFunction` also needs the initial 
"template" (assuming the whole function isn't an implicit parameter).

In theory, a `Space` and a `State` are just a dictionary and list assigning
Boolean values to `VariableId`. However, we still use them because they have 
a few nice semantic properties.

## General Boolean / regulatory networks

### `VariableId` and `Variable`

Type alias `VariableIdIterator` is not needed, as we mostly just copy a vector
of IDs.

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
            <td><code>VariableId::__init__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::hash</code></td>
            <td><code>VariableId::__hash__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::to_string</code></td>
            <td><code>VariableId::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>VariableId::__repr__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::eq</code></td>
            <td><code>VariableId::__eq__</code></td>
        </tr>
        <tr>
            <td rowspan="4"><code>VariableId::cmp</code></td>
            <td><code>VariableId::__lt__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::__le__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::__gt__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::__ge__</code></td>
        </tr>
        <tr>
            <td rowspan="3"><code>VariableId::to_index</code></td>
            <td><code>VariableId::into_index</code></td>
        </tr>
        <tr>
            <td><code>VariableId::__index__</code></td>
        </tr>
        <tr>
            <td><code>VariableId::__int__</code></td>
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
            <td><code>Variable</code> constructor</td>
            <td><code>Variable::__init__</code></td>
        </tr>
        <tr>
            <td><code>Variable::hash</code></td>
            <td><code>Variable::__hash__</code></td>
        </tr>
        <tr>
            <td><code>Variable::to_string</code></td>
            <td><code>Variable::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Variable::__repr__</code></td>
        </tr>
        <tr>
            <td><code>Variable::eq</code></td>
            <td><code>Variable::__eq__</code></td>
        </tr>
        <tr>
            <td><code>Variable::get_name</code></td>
            <td><code>Variable::name</code> (read-only property)</td>
        </tr>
    </tbody>
</table>

### `ParameterId` and `Parameter`

Type alias `ParameterIdIterator` is not needed, as we mostly just copy a vector
of IDs.

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
            <td><code>ParameterId::__init__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::hash</code></td>
            <td><code>ParameterId::__hash__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::to_string</code></td>
            <td><code>ParameterId::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>ParameterId::__repr__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::eq</code></td>
            <td><code>ParameterId::__eq__</code></td>
        </tr>
        <tr>
            <td rowspan="4"><code>ParameterId::cmp</code></td>
            <td><code>ParameterId::__lt__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::__le__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::__gt__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::__ge__</code></td>
        </tr>
        <tr>
            <td rowspan="3"><code>ParameterId::to_index</code></td>
            <td><code>ParameterId::into_index</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::__index__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::__int__</code></td>
        </tr>
        <tr>
            <td><code>ParameterId::try_from_usize</code></td>
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
            <td><code>Parameter</code> constructor</td>
            <td><code>Parameter::__init__</code></td>
        </tr>
        <tr>
            <td><code>Parameter::hash</code></td>
            <td><code>Parameter::__hash__</code></td>
        </tr>
        <tr>
            <td><code>Parameter::to_string</code></td>
            <td><code>Parameter::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Parameter::__repr__</code></td>
        </tr>
        <tr>
            <td><code>Parameter::eq</code></td>
            <td><code>Parameter::__eq__</code></td>
        </tr>
        <tr>
            <td><code>Parameter::get_name</code></td>
            <td><code>Parameter::name</code> (read-only property)</td>
        </tr>
        <tr>
            <td><code>Parameter::get_arity</code></td>
            <td><code>Parameter::arity</code> (read-only property)</td>
        </tr>
    </tbody>
</table>

### `Regulation`

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
            <td><code>Regulation</code> constructor</td>
            <td rowspan="2"><code>Regulation::__init__</code></td>
        </tr>
        <tr>
            <td><code>Regulation::try_from_string</code></td>
        </tr>
        <tr>
            <td><code>Regulation::hash</code></td>
            <td><code>Regulation::__hash__</code></td>
        </tr>
        <tr>
            <td><code>Regulation::to_string</code></td>
            <td><code>Regulation::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Regulation::__repr__</code></td>
        </tr>
        <tr>
            <td><code>Regulation::eq</code></td>
            <td><code>Regulation::__eq__</code></td>
        </tr>
        <tr>
            <td><code>Regulation::get_regulator</code></td>
            <td><code>Regulation::source</code> (read-only property)</td>
        </tr>
        <tr>
            <td><code>Regulation::get_target</code></td>
            <td><code>Regulation::target</code> (read-only property)</td>
        </tr>
        <tr>
            <td><code>Regulation::get_monotonicity</code></td>
            <td><code>Regulation::monotonicity</code> (read-only property)</td>
        </tr>
        <tr>
            <td><code>Regulation::is_observable</code></td>
            <td><code>Regulation::observable</code> (read-only property)</td>
        </tr>
    </tbody>
</table>

### `Space`

A space is simply a dictionary mapping `VariableId` to a `bool` value with explicit 
support for missing variables (where it returns `None`). Note that we probably can't 
implement this directly using the Rust counterpart, so this is in fact a completely
separate structure that just happens to have a similar API. Conceptually, it is
similar to `BddPartialValuation` over a different domain.

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
            <td><code>Space::new</code></td>
            <td rowspan="2"><code>Space::__init__</code></td>
        </tr>
        <tr>
            <td><code>Space::from_values</code></td>
        </tr>
        <tr>
            <td><code>Space::hash</code></td>
            <td><code>Space::__hash__</code></td>
        </tr>
        <tr>
            <td><code>Space::to_string</code></td>
            <td><code>Space::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Space::__repr__</code></td>
        </tr>
        <tr>
            <td><code>Space::eq</code></td>
            <td><code>Space::__eq__</code></td>
        </tr>
        <tr>
            <td><code>Space::index</code></td>
            <td><code>Space::__getitem__</code></td>
        </tr>
        <tr>
            <td rowspan="2"><code>Space::index_mut</code></td>
            <td><code>Space::__setitem__</code></td>
        </tr>
        <tr>
            <td><code>Space::__delitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Space::__contains__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Space::__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Space::__iter__</code></td>
        </tr>
        <tr>
            <td><code>Space::clone</code></td>
            <td><code>Space::copy</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other methods</td></tr>
        <tr>
            <td rowspan="2"><code>Space::to_values</code></td>
            <td><code>Space::to_dict</code></td>
        </tr>
        <tr>
            <td><code>Space::to_list</code></td>
        </tr>
        <tr>
            <td><code>Space::intersect</code></td>
            <td><code>Space::intersect</code></td>
        </tr>
        <tr>
            <td><code>Space::partial_cmp</code></td>
            <td><code>Space::is_subset</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>Space::support_set</code></td>
        </tr>
        <tr>
            <td><code>Space::count_any</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Space::is_trap_space</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

### `State`

The logic behind this structure is similar to `Space`, but in this case there
isn't a concrete Rust counterpart. However, the API is again similar to
`BddValuation` and the whole thing more or less behaves like a list of immutable 
length.

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
            <td><code>State::__init__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__len__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__getitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__setitem__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__eq__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::to_list</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::to_dict</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>State::to_space</code></td>
        </tr>
    </tbody>
</table>

### `RegulatoryGraph`

Currently, we don't export `RegulationIterator`. A copy of the data is 
returned instead.

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
            <td rowspan="2"><code>RegulatoryGraph::__init__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::try_from(&str)</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::try_from_string_regulations</code></td>
            <td></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::__repr__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::eq</code></td>
            <td><code>RegulatoryGraph::__eq__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::index</code></td>
            <td rowspan="3"><code>RegulatoryGraph::__getitem__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::get_variable</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::find_regulation</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>RegulatoryGraph::__contains__</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::clone</code></td>
            <td><code>RegulatoryGraph::copy</code></td>
        </tr>
        <tr><td colspan="2" align="center">Introspection</td></tr>
        <tr>
            <td><code>RegulatoryGraph::num_vars</code></td>
            <td><code>RegulatoryGraph::var_count</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::variables</code></td>
            <td><code>RegulatoryGraph::variables</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::regulations</code></td>
            <td><code>RegulatoryGraph::regulations</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::find_variable</code></td>
            <td><code>RegulatoryGraph::id_of</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::regulators</code></td>
            <td rowspan="2"><code>RegulatoryGraph::regulators</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::regulators_transitive</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::targets</code></td>
            <td rowspan="2"><code>RegulatoryGraph::targets</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::targets_transitive</code></td>
        </tr>
        <tr><td colspan="2" align="center">Modifications</td></tr>
        <tr>
            <td><code>RegulatoryGraph::add_regulation</code></td>
            <td rowspan="2"><code>RegulatoryGraph::add_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::add_string_regulation</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::set_variable_name</code></td>
            <td><code>RegulatoryGraph::set_variable_name</code></td>
        </tr>
        <tr><td colspan="2" align="center">Conversion methods</td></tr>
        <tr>
            <td><code>RegulatoryGraph::to_string</code></td>
            <td><code>RegulatoryGraph::to_aeon</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::to_dot</code></td>
            <td><code>RegulatoryGraph::to_dot_string</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::write_as_dot</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Algorithms</td></tr>
        <tr>
            <td><code>RegulatoryGraph::feedback_vertex_set</code></td>
            <td rowspan="2"><code>RegulatoryGraph::feedback_vertex_set</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::parity_feedback_vertex_set</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::independent_cycles</code></td>
            <td rowspan="2"><code>RegulatoryGraph::independent_cycles</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::independent_parity_cycles</code></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::strongly_connected_components</code></td>
            <td rowspan="3"><code>RegulatoryGraph::strongly_connected_components
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
            <td><code>RegulatoryGraph::shortest_cycle</code></td>
            <td rowspan="2"><code>RegulatoryGraph::shortest_cycle</code
></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::shortest_parity_cycle</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>RegulatoryGraph::is_valid_name</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>RegulatoryGraph::get_variable_name</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

### `UpdateFunction` (`FnUpdate`)

Update function API is modelled similarly as the `BooleanExpression`. However,
instead of using string names, it uses `VariableId` and `ParameterId` types.

Due to this decision, it is not possible to create an `UpdateFunction` directly
from a string expression. However, it can be created assuming a suitable `BooleanNetwork`
is given.

Update functions can be also created using the "infix operators".

Due to the API design mimicking `BooleanExpression`, there is no longer use for `BinaryOp`.

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
            <td><code>FnUpdate::try_from_expression</code></td>
            <td><code>UpdateFunction::__init__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::eq</code></td>
            <td><code>UpdateFunction::__eq__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::hash</code></td>
            <td><code>UpdateFunction::__hash__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::__repr__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::contains_variable</code></td>
            <td rowspan="2"><code>UpdateFunction::__contains__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::contains_parameter</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::eval_in_space</code></td>
            <td rowspan="2"><code>UpdateFunction::__call__</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::evaluate</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern constructors</td></tr>
        <tr>
            <td><code>FnUpdate::Const</code></td>
            <td rowspan="3"><code>UpdateFunction::mk_const</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_true</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_false</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Var</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Param</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Not</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_not</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::negation</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Binary</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_and</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::and</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Binary</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_or</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::or</code></td>
        </tr>
                <tr>
            <td><code>FnUpdate::Binary</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_imp</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::implies</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Binary</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_iff</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::iff</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::Binary</code></td>
            <td rowspan="2"><code>UpdateFunction::mk_xor</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::xor</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::mk_binary</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern tests</td></tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_param</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_literal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::is_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern destructors</td></tr>
        <tr>
            <td><code>FnUpdate::as_const</code></td>
            <td><code>UpdateFunction::as_const</code></td>
        </tr>   
        <tr>
            <td><code>FnUpdate::as_var</code></td>
            <td><code>UpdateFunction::as_var</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_param</code></td>
            <td><code>UpdateFunction::as_param</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::as_not</code></td>
            <td><code>UpdateFunction::as_not</code></td>
        </tr>
        <tr>
            <td rowspan="5"><code>FnUpdate::as_binary</code></td>
            <td><code>UpdateFunction::as_and</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::as_or</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::as_imp</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::as_iff</code></td>
        </tr>
        <tr>
            <td><code>UpdateFunction::as_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::as_literal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>UpdateFunction::as_binary</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>FnUpdate::to_string</code></td>
            <td><code>UpdateFunction::to_string</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::collect_arguments</code></td>
            <td><code>UpdateFunction::support_variables</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::collect_parameters</code></td>
            <td><code>UpdateFunction::support_parameters</code></td>
        </tr>
        <tr>
            <td><code>FnUpdate::to_and_or_normal_form</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::distribute_negation</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::is_specialisation_of</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::substitute</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::walk_postorder</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FnUpdate::build_from_bdd</code></td>
            <td>(part of <code>SymbolicContext</code>)</td>
        </tr>
    </tbody>
</table>

### `BooleanNetwork`

The `BooleanNetwork` class inherits from `RegulatoryNetwork`. However, some of the
methods need to be overriden to properly support the declared functionality. Not
all overrides are a part of this table, but all Rust functions that implement
a functionality provided by `RegulatoryNetwork` are listed.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Constructors and conversions</td></tr>
        <tr>
            <td><code>BooleanNetwork::try_from(&str)</code></td>
            <td><code>BooleanNetwork::from_aeon</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_string</code></td>
            <td><code>BooleanNetwork::to_aeon</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_bnet</code></td>
            <td><code>BooleanNetwork::from_bnet</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_bnet</code></td>
            <td><code>BooleanNetwork::to_bnet</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_sbml</code></td>
            <td rowspan="2"><code>BooleanNetwork::from_sbml</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_sbml_strict</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::to_sbml</code></td>
            <td rowspan="2"><code>BooleanNetwork::to_sbml</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::write_as_sbml</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::try_from_file</code></td>
            <td><code>BooleanNetwork::from_file</code></td>
        </tr>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td><code>BooleanNetwork::new</code></td>
            <td><code>BooleanNetwork::__init__</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::eq</code></td>
            <td><code>BooleanNetwork::__eq__</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::index</code></td>
            <td rowspan="4"><code>BooleanNetwork::__getitem__</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::find_parameter</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_parameter</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_variable</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork::__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork::__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork::__contains__</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::clone</code></td>
            <td><code>BooleanNetwork::copy</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::as_graph</code></td>
            <td rowspan="2"><code>BooleanNetwork::copy_graph</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::as_graph_mut</code></td>
        </tr>
        <tr><td colspan="2" align="center">Introspection</td></tr>
        <tr>
            <td><code>BooleanNetwork::num_vars</code></td>
            <td>(inherited)</td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::num_parameters</code></td>
            <td><code>BooleanNetwork::parameter_count</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::num_implicit_parameters</code></td>
            <td><code>BooleanNetwork::implicit_parameter_count</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::parameters</code></td>
            <td><code>BooleanNetwork::parameters</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::implicit_parameters</code></td>
            <td><code>BooleanNetwork::implicit_parameters</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_parameter</code></td>
            <td><code>BooleanNetwork::add_parameter</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_update_function</code></td>
            <td><code>BooleanNetwork::get_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::set_update_function</code></td>
            <td rowspan="3"><code>BooleanNetwork::set_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::add_string_update_function</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>BooleanNetwork::has_update_function</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::variables</code></td>
            <td>(inherited)</td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::targets</code></td>
            <td>(inherited)</td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::regulators</code></td>
            <td>(inherited)</td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td><code>BooleanNetwork::inline_inputs</code></td>
            <td><code>BooleanNetwork::inline_inputs</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::infer_valid_graph</code></td>
            <td><code>BooleanNetwork::infer_regulatory_graph</code></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::make_witness</code></td>
            <td>(deprecated)</td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::make_witness_for_valuation</code></td>
            <td>(deprecated)</td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::is_valid_name</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::get_variable_name</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>BooleanNetwork::parameter_appears_in</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

## Algorithmic functionality

This mainly includes "basic" algorithms that may (at some point) be delegated
to separate crates, but for now are part of `lib-param-bn`.

### `FixedPoints`

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center">Static methods</td></tr>
        <tr>
            <td><code>FixedPoints::naive_symbolic</code></td>
            <td><code>FixedPoints::naive_symbolic</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic</code></td>
            <td><code>FixedPoints::symbolic</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_vertices</code></td>
            <td><code>FixedPoints::symbolic_vertices</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_colors</code></td>
            <td><code>FixedPoints::symbolic_colors</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic</code></td>
            <td><code>FixedPoints::symbolic</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_projection</code></td>
            <td><code>FixedPoints::symbolic_projection</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_merge</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::symbolic_iterator</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>FixedPoints::solver_iterator</code></td>
            <td><code>FixedPoints::solver</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::solver_vertex_iterator</code></td>
            <td><code>FixedPoints::solver_vertices</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::solver_color_iterator</code></td>
            <td><code>FixedPoints::solver_colors</code></td>
        </tr>
        <tr>
            <td><code>FixedPoints::make_fixed_points_solver</code></td>
            <td></td>
        </tr>
    </tbody>
</table>