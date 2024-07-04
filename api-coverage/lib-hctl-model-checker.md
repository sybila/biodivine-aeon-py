# Biodivine `lib-hctl-model-checker` API coverage

This document should be up-to-date with `lib-hctl-model-checker` version `0.2.2`.

Overall, the library exposes a lot of methods that could be considered "internal",
so a lot of the low-level features are not exposed in the Python bindings.

### `preprocessing` module

First, the general function definitions plus enum types that are translated to Python `Literal` types.

<!--suppress XmlDeprecatedElement, HtmlDeprecatedAttribute -->
<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center"><code>utils</code></td></tr>
        <tr>
            <td><code>check_props_and_rename_vars</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center"><code>tokenizer</code></td></tr>
        <tr>
            <td><code>HctlToken::Atom</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlToken::Unary</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlToken::Binary</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlToken::Hybrid</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlToken::Tokens</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>print_tokens</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>try_tokenize_extended_formula</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>try_tokenize_formula</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center"><code>read_inputs</code></td></tr>
        <tr>
            <td><code>load_and_parse_bn_model</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>load_formulae</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center"><code>operator_enums</code></td></tr>
        <tr>
            <td><code>HybridOp::Bind</code></td>
            <td rowspan="4"><code>HybridOperator</code></td>
        </tr>
        <tr>
            <td><code>HybridOp::Jump</code></td> 
        </tr>
        <tr>
            <td><code>HybridOp::Exists</code></td>
        </tr>
        <tr>
            <td><code>HybridOp::Forall</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Not</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Ex</code></td>
            <td rowspan="6"><code>TemporalUnaryOperator</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Ax</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Ef</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Af</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Eg</code></td>
        </tr>
        <tr>
            <td><code>UnaryOp::Ag</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::And</code></td>
            <td rowspan="5"><code>BinaryOperator</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Or</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Xor</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Imp</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Iff</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Eu</code></td>
            <td rowspan="4"><code>TemporalBinaryOperator</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Au</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Ew</code></td>
        </tr>
        <tr>
            <td><code>BinaryOp::Aw</code></td>
        </tr>
        <tr>
            <td><code>Atomic::Prop</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Atomic::Var</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Atomic::True</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Atomic::False</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>Atomic::WildCardProp</code></td>
            <td></td>
        </tr>
        <tr><td colspan="2" align="center"><code>parser</code></td></tr>
        <tr>
            <td><code>parse_and_minimize_extended_formula</code></td>
            <td rowspan="4"><code>HctlFormula.__init__</code></td>
        </tr>
        <tr>
            <td><code>parse_and_minimize_hctl_formula</code></td>
        </tr>
        <tr>
            <td><code>parse_extended_formula</code></td>
        </tr>
        <tr>
            <td><code>parse_hctl_formula</code></td>
        </tr>
        <tr>
            <td><code>parse_hctl_tokens</code></td>
            <td></td>
        </tr>
    </tbody>
</table>

Finally, the `HctlFormula` structure, which has been adapted to more closely resemble 
the current API of `BooleanExpression` and `UpdateFunction`.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr><td colspan="2" align="center"><code>NodeType</code></td></tr>
        <tr>
            <td><code>NodeType::TerminalNode</code></td>
            <td rowspan="4">Loosely map to <code>mk_*</code> methods.</td>
        </tr>
        <tr>
            <td><code>NodeType::UnaryNode</code></td>
        </tr>
        <tr>
            <td><code>NodeType::BinaryNode</code></td>
        </tr>
        <tr>
            <td><code>NodeType::HybridNode</code></td>
        </tr>
        <tr><td colspan="2" align="center">Special methods</td></tr>
        <tr>
            <td>(see above)</td>
            <td><code>HctlFormula.__init__</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::new</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::hash</code></td>
            <td><code>HctlFormula.__hash__</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::eq</code></td>
            <td><code>HctlFormula.__richcmp__</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::to_string</code></td>
            <td><code>HctlFormula.__str__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.__repr__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.__getnewargs__</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.__root__</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::subform_str</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::height</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::node_type</code></td>
            <td>Implemented as patterns.</td>
        </tr>
        <tr><td colspan="2" align="center">Pattern constructors</td></tr>
        <tr>
            <td rowspan="5"><code>HctlTreeNode::mk_hybrid_node</code></td>
            <td><code>HctlFormula.mk_hybrid</code></td>
        </tr>
        <tr>
            <td><code>HctlFormula.mk_exists</code></td>
        </tr>
        <tr>
            <td><code>HctlFormula.mk_forall</code></td>
        </tr>
        <tr>
            <td><code>HctlFormula.mk_bind</code></td>
        </tr>
        <tr>
            <td><code>HctlFormula.mk_jump</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_unary_node</code></td>
            <td rowspan="2"><code>HctlFormula.mk_temporal</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_binary_node</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_boolean</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_exist_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_all_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_exist_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_all_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_exist_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_all_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_exist_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_all_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_exist_weak_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.mk_all_weak_until</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_var_node</code></td>
            <td><code>HctlFormula.mk_state_var</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_prop_node</code></td>
            <td><code>HctlFormula.mk_network_var</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_constant_node</code></td>
            <td><code>HctlFormula.mk_const</code></td>
        </tr>
        <tr>
            <td><code>HctlTreeNode::mk_wild_card_node</code></td>
            <td><code>HctlFormula.mk_extended_prop</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern tests</td></tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_hybrid</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_temporal</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_temporal_unary</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_temporal_binary</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_boolean</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_state_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_network_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_extended_prop</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exists</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_forall</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_bind</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_jump</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exist_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_all_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exist_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_all_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exist_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_all_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exist_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_all_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_exist_weak_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.is_all_weak_until</code></td>
        </tr>
        <tr><td colspan="2" align="center">Pattern destructors</td></tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_hybrid</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_temporal_unary</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_temporal_binary</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_boolean</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_state_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_network_var</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_const</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_extended_prop</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exists</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_forall</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_bind</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_jump</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_not</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_and</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_or</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_imp</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_iff</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_xor</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exist_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_all_next</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exist_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_all_future</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exist_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_all_global</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exist_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_all_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_exist_weak_until</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.as_all_weak_until</code></td>
        </tr>
        <tr><td colspan="2" align="center">Other</td></tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.children</code></td>
        </tr>
        <tr>
            <td></td>
            <td><code>HctlFormula.operator</code></td>
        </tr>
    </tbody>
</table>

### `result_print`, `analysis`, `postprocessing` and `evaluation` modules

These modules are not mapped to the Python API. Modules `result_print` and `analysis` are only
relevant for an "executable" mode where the results are printed directly to standard output.

Module `evaluation` is the internal implementation of the algorithm that is largely too technical to
use as a high-level API from Python.

Module `postprocessing` is superseded by the `transfer_from` functions that are
already available on `AsynchronousGraph` objects.

### `mc_utils` module

These functions are mostly added as methods to the more appropriate types, since now we have them in one
package, and we can do that.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>check_hctl_var_support</code></td>
            <td><code>HctlFormula.is_compatible_with</code></td>
        </tr>
        <tr>
            <td><code>collect_unique_hctl_vars</code></td>
            <td><code>HctlFormula.used_state_variables</code></td>
        </tr>
        <tr>
            <td><code>collect_unique_wild_card_props</code></td>
            <td><code>HctlFormula.used_extended_properties</code></td>
        </tr>
        <tr>
            <td><code>get_extended_symbolic_graph</code></td>
            <td><code>AsynchronousGraph.mk_for_model_checking</code></td>
        </tr>  
    </tbody>
</table>

### `model_checking` module

These are mostly mapped to a singleton `ModelChecking` object. We
ignore the methods that perform automatic sanitization, because we expect
the user to know what graph/context they are using. Automatic sanitization
mostly made sense in the original library where the expectation was that the
results will be written to some output file and the context will be lost.

<table>
    <thead>
        <tr>
            <th>Rust Member</th>
            <th>Python Member</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>model_check_extended_formula_dirty</code></td>
            <td><code>ModelChecking.verify</code></td>
        </tr>
        <tr>
            <td><code>model_check_formula_dirty</code></td>
            <td><code>ModelChecking.verify</code></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_extended_formulae_dirty</code></td>
            <td><code>ModelChecking.verify</code></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_formulae_dirty</code></td>
            <td><code>ModelChecking.verify</code></td>
        </tr>
        <tr>
            <td><code>model_check_extended_formula</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_formula</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_formula_unsafe_ex</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_extended_formulae</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_formulae</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_trees</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_multiple_trees_dirty</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_tree</code></td>
            <td></td>
        </tr>
        <tr>
            <td><code>model_check_tree_dirty</code></td>
            <td></td>
        </tr>
    </tbody>
</table>