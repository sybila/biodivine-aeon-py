use std::cmp::max;
use std::collections::HashMap;

use biodivine_hctl_model_checker::mc_utils::get_extended_symbolic_graph;
use biodivine_hctl_model_checker::model_checking::model_check_multiple_extended_formulae_dirty;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColors, SymbolicContext as RsSymbolicContext,
};
use biodivine_pbn_control::control::PhenotypeOscillationType;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyDict, PyList};
use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyRef, PyResult, Python};

use crate::bindings::bn_classifier::class::{extend_map, Class};
use crate::bindings::lib_hctl_model_checker::hctl_formula::HctlFormula;
use crate::bindings::lib_param_bn::algorithms::attractors::Attractors;
use crate::bindings::lib_param_bn::algorithms::reachability::Reachability;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::pbn_control::extract_phenotype_type;
use crate::internal::classification::load_inputs::load_classification_archive;
use crate::internal::classification::write_output::build_classification_archive;
use crate::internal::scc::{Behaviour, Classifier};
use crate::{runtime_error, throw_runtime_error, throw_type_error, AsNative};

/// An "algorithm object" that groups all methods related to the classification of various
/// model properties.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Classification {
    _dummy: (),
}

#[pymethods]
impl Classification {
    /// Extend an existing `classification` dictionary in such a way that every color
    /// in the `colors` set appears in a `Class` with the specified `features`.
    ///
    /// For example: Extending `{ ['a']: [1,2,3], ['b']: [4,5,6] }` with `'a': [3,4]` results in
    /// `{ `a`: [1,2,3], ['b']: [5,6], ['a','b']: [4] }`.
    ///
    /// This does not "increase" the number of times a feature appears in a class, it merely
    /// creates new classes if the feature is not present.
    #[staticmethod]
    pub fn ensure(
        classification: HashMap<Class, ColorSet>,
        features: Bound<'_, Class>,
        colors: ColorSet,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        let mut new_classification = HashMap::new();
        for (cls, set) in &classification {
            // Save the unaffected colors with the existing class.
            let rest = set.minus(&colors);
            if !rest.is_empty() {
                extend_map(&mut new_classification, cls, rest);
            }
            // Create a new class for the intersection.
            let both = set.intersect(&colors);
            if !both.is_empty() {
                let new_cls = cls.ensure(features.as_any())?;
                extend_map(&mut new_classification, &new_cls, both);
            }
        }
        Ok(new_classification)
    }

    /// Extend an existing `classification` dictionary in such a way that every color
    /// in the `colors` set has an additional features according to the specified `Class`.
    ///
    /// For example: Extending `{ ['a']: [1,2,3], ['b']: [4,5,6] }` with `'a': [3,4]` results in
    /// `{ `a`: [1,2], ['b']: [5,6], ['a','a']: [3], ['a','b']: [4] }`.
    ///
    /// In other words, compared to `Class.classification_ensure`, this does "increase" the number
    /// of times a feature appears in a class.
    #[staticmethod]
    pub fn append(
        classification: HashMap<Class, ColorSet>,
        features: Bound<'_, Class>,
        colors: ColorSet,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        let mut new_classification = HashMap::new();
        for (cls, set) in &classification {
            // Save the unaffected colors with the existing class.
            let rest = set.minus(&colors);
            if !rest.is_empty() {
                extend_map(&mut new_classification, cls, rest);
            }
            // Create a new class for the intersection.
            let both = set.intersect(&colors);
            if !both.is_empty() {
                let new_cls = cls.append(features.as_any())?;
                extend_map(&mut new_classification, &new_cls, both);
            }
        }
        Ok(new_classification)
    }

    /// Read the list of *dynamic assertions* from `.aeon` model annotations.
    ///
    /// An assertion typically encodes a `HctlFormula` that must be satisfied by all
    /// the relevant interpretations of a partially specified model.
    ///
    /// The argument is either a `ModelAnnotation` dictionary, a path to an existing model
    /// file from which the annotations are extracted, or a multi-line string content that
    /// directly represents the annotations.
    ///
    /// Assertions appear as a `#! dynamic_assertion: CONTENT` annotation
    /// comments in the `.aeon` file.
    #[staticmethod]
    pub fn read_dynamic_assertions(
        py: Python,
        annotation_source: &Bound<'_, PyAny>,
    ) -> PyResult<Vec<String>> {
        // Based on the old read_model_assertions method that only works on the native annotation object.
        let annotations = extract_annotations(py, annotation_source)?;
        let Some(list) = annotations.__getitem__("dynamic_assertion").get_value(py) else {
            return Ok(Vec::new());
        };
        Ok(list.lines().map(|it| it.to_string()).collect())
    }

    /// Read the list of *dynamic properties* from `.aeon` model annotations.
    ///
    /// A property typically encodes a `HctlFormula` that is of interest in a particular model,
    /// but is not necessarily satisfied by all relevant interpretations of a model. Each property
    /// is identified by a name (first item in the tuple).
    ///
    /// The argument is either a `ModelAnnotation` dictionary, or a path to an existing model
    /// file from which the annotations are extracted, or a multi-line string content that
    /// directly represents the annotations.
    ///
    /// Properties appear as a `#! dynamic_property: NAME: CONTENT` annotation
    /// comments in the `.aeon` file.
    #[staticmethod]
    pub fn read_dynamic_properties(
        py: Python,
        annotation_source: &Bound<'_, PyAny>,
    ) -> PyResult<Vec<(String, String)>> {
        let annotations = extract_annotations(py, annotation_source)?;
        let property_node = annotations.__getitem__("dynamic_property");
        let mut properties = Vec::new();
        for (name, child) in property_node.items(py) {
            if !child.items(py).is_empty() {
                // TODO:
                //  This might actually be a valid (if ugly) way for adding extra meta-data to
                //  properties, but let's forbid it for now and we can enable it later if
                //  there is an actual use for it.
                return throw_runtime_error(format!("Property `{name}` contains nested values."));
            }
            let Some(value) = child.get_value(py) else {
                return throw_runtime_error(format!("Found empty dynamic property `{name}`."));
            };
            if value.lines().count() > 1 {
                return throw_runtime_error(format!("Found multiple properties named `{name}`."));
            }
            properties.push((name.clone(), value.clone()));
        }
        // Sort alphabetically to avoid possible non-determinism down the line.
        properties.sort_by(|(x, _), (y, _)| x.cmp(y));
        Ok(properties)
    }

    /// Write the provided *dynamic assertions* into a `ModelAnnotation` dictionary. Note that
    /// this method does not modify any assertions that may already exist in the annotation
    /// dictionary.
    ///
    /// Assertions appear as a `#! dynamic_assertion: CONTENT` annotation
    /// comments in the `.aeon` file.
    #[staticmethod]
    pub fn write_dynamic_assertions(
        py: Python,
        annotations: &ModelAnnotation,
        assertions: Vec<String>,
    ) -> PyResult<()> {
        let assertions_node = annotations.__getitem__("dynamic_assertion");
        let mut current = assertions_node.get_value(py).unwrap_or_default();
        for assertion in assertions {
            if !current.is_empty() {
                current.push('\n')
            }
            current.push_str(assertion.as_str())
        }
        assertions_node.set_value(py, Some(current));
        Ok(())
    }

    /// Write the provided *dynamic properties* into a `ModelAnnotation` dictionary. Note that
    /// this method does not modify any properties that may already exist in the annotation
    /// dictionary. If a property of the same name already exists, the method fails with
    /// a `RuntimeError`.
    ///
    /// Properties appear as a `#! dynamic_property: NAME: CONTENT` annotation
    /// comments in the `.aeon` file.
    #[staticmethod]
    pub fn write_dynamic_properties(
        py: Python,
        annotations: &ModelAnnotation,
        properties: Vec<(String, String)>,
    ) -> PyResult<()> {
        let properties_node = annotations.__getitem__("dynamic_property");
        for (name, prop) in properties {
            let prop_node = properties_node.__getitem__(name.as_str());
            if prop_node.get_value(py).is_some() {
                return throw_runtime_error(format!("Property `{}` is already set.", name));
            } else {
                prop_node.set_value(py, Some(prop));
            }
        }
        Ok(())
    }

    /// Save the classification results into a `.zip` archive that can be analyzed by
    /// the [BN Classifier](https://github.com/sybila/biodivine-bn-classifier/).
    ///
    /// The `annotations` dictionary is optional. By default, all annotations that are already
    /// associated with the `BooleanNetwork` instance are saved. However, you can use this argument
    /// to add additional annotations. These could include the HCTL properties and assertions
    /// that were used to create the `classification` dictionary, assuming they are not part of
    /// network annotations already.
    ///
    /// Note that this method will automatically sanitize the `ColorSet` objects such that they
    /// use the "default" symbolic encoding for the provided `network`.
    ///
    #[staticmethod]
    #[pyo3(signature = (path, network, classification, annotations = None))]
    pub fn save_classification(
        py: Python,
        path: String,
        network: PyRef<'_, BooleanNetwork>,
        classification: HashMap<Class, ColorSet>,
        annotations: Option<ModelAnnotation>,
    ) -> PyResult<()> {
        let ctx = RsSymbolicContext::new(network.as_native()).map_err(runtime_error)?;

        let classes = classification
            .into_iter()
            .map(|(k, v)| {
                if let Some(bdd) =
                    ctx.transfer_from(v.as_native().as_bdd(), v.__ctx__().get().as_native())
                {
                    Ok((k.as_serial_string(), GraphColors::new(bdd, &ctx)))
                } else {
                    throw_runtime_error(
                        "One of the class sets is not compatible with the given network.",
                    )
                }
            })
            .collect::<PyResult<HashMap<_, _>>>()?;

        let mut aeon_file = BooleanNetwork::to_aeon(network, py);
        if let Some(annotations) = annotations {
            let ann_string = annotations.__str__(py);
            aeon_file.push('\n');
            aeon_file.push_str(ann_string.as_str());
        }

        let Err(e) = build_classification_archive(classes, path.as_str(), aeon_file.as_str())
        else {
            return Ok(());
        };

        throw_runtime_error(format!("Cannot write archive: {}", e))
    }

    /// Load a [BN Classifier](https://github.com/sybila/biodivine-bn-classifier/) archive
    /// into a usable representation. The result includes:
    ///
    ///  - The original `BooleanNetwork` for which the classification is valid.
    ///  - A classification mapping from (collections of) properties to sets of
    ///    network interpretations.
    ///  - A `ModelAnnotation` object that can (optionally) contain the properties that
    ///    were used to generate the classification.
    ///
    #[staticmethod]
    pub fn load_classification(
        py: Python,
        path: String,
    ) -> PyResult<(
        Py<BooleanNetwork>,
        HashMap<Class, ColorSet>,
        ModelAnnotation,
    )> {
        let (classes, model) = load_classification_archive(path).map_err(runtime_error)?;
        let annotations = ModelAnnotation::from_aeon(py, model.as_str())?;
        let network = BooleanNetwork::from_aeon(py, model.as_str())?;
        let ctx = SymbolicContext::new(py, network.clone(), None)?;
        let ctx = Py::new(py, ctx)?;
        let classification = classes
            .into_iter()
            .map(|(k, v)| {
                (
                    Class::from_serial_string(k),
                    ColorSet::mk_native(ctx.clone(), v),
                )
            })
            .collect::<HashMap<_, _>>();
        Ok((network, classification, annotations))
    }

    /// Classify the interpretations (colors) of the provided `component` based on their long-term
    /// behavior. That is:
    ///
    ///  - `stable`: the component is a single state;
    ///  - `cycle`: all states in the component form a single (deterministic) cycle (each state
    ///    has exactly one successor).
    ///  - `complex`: any other set of states that has a non-trivial structure.
    ///
    /// Note that this method is primarily intended to be used with attractors, but can be
    /// (in theory) also applied to any colored set of vertices. However, in such case, the
    /// result will typically be the `complex` behavior class.
    ///
    /// However, the properties described above are only tested in a subgraph induced by the
    /// vertices in the given `component`. Hence, you can use the method to at least distinguish
    /// between cycles and other complex components, or to quickly identify colors for which
    /// a component is trivial (i.e. `stable`).
    ///
    #[staticmethod]
    pub fn classify_long_term_behavior(
        graph: &AsynchronousGraph,
        component: &ColoredVertexSet,
    ) -> HashMap<Class, ColorSet> {
        let classes = Classifier::classify_component(component.as_native(), graph.as_native());
        classes
            .into_iter()
            .map(|(k, v)| {
                let k = vec![encode_behavior(k)];
                (
                    Class::new_native(k),
                    ColorSet::mk_native(graph.symbolic_context(), v),
                )
            })
            .collect()
    }

    /// Perform a full classification of attractor behavior in the given `AsynchronousGraph`.
    ///
    /// This is a generalization of `Classification.classify_long_term_behavior` in the sense
    /// that the result covers all attractors, not just one component. Note that the `Class`
    /// instances produced by this process *count* the number of attractors of each behavior type.
    ///
    /// If attractors are already known, or you wish to only consider a subset of attractors,
    /// you can provide them through the optional `attractors` argument.
    ///
    /// Note that you can achieve similar results using
    /// `Classification.classify_long_term_behavior` and `Classification.ensure`
    /// (or `Classification.append`). However, this process is not limited to attractors
    /// and can be potentially combined with other features (like HCTL properties).
    #[staticmethod]
    #[pyo3(signature = (graph, attractors = None))]
    pub fn classify_attractor_bifurcation(
        py: Python,
        graph: &AsynchronousGraph,
        attractors: Option<Vec<ColoredVertexSet>>,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        let attractors = if let Some(attractors) = attractors {
            attractors
        } else {
            Attractors::attractors(graph, None, None, py)?
        };

        let attractors = attractors
            .into_iter()
            .map(|it| it.as_native().clone())
            .collect::<Vec<_>>();

        let scc_classifier = Classifier::new(graph.as_native());
        for attr in attractors {
            scc_classifier.add_component(attr, graph.as_native());
        }
        let classification = scc_classifier.export_result();

        Ok(classification
            .into_iter()
            .map(|(k, v)| {
                let items = k.0.into_iter().map(encode_behavior).collect::<Vec<_>>();
                (
                    Class::new_native(items),
                    ColorSet::mk_native(graph.symbolic_context(), v),
                )
            })
            .collect())
    }

    /// Classify the *individual attractors* of an `AsynchronousGraph` according to their affinity
    /// to biological phenotypes.
    ///
    /// This is similar to `Classification.classify_phenotypes`, but it actually returns
    /// per-attractor results instead of lumping all attractors of one phenotype into
    /// a single class. This results in a sort-of nested classification, where each class
    /// consists of entries for individual attractors, but each entry is itself a class
    /// that describes the phenotypes of that particular attractor.
    ///
    /// For example, given phenotypes `"a"` and `"b"`, the result could contain a class
    /// `Class(["Class(['a'])", "Class(['b'])", "Class(['a', 'b'])"])`. This means that there
    /// are three attractors: One with only the "a" phenotype, one with only the "b" phenotype,
    /// and one with both phenotypes. Note that the "inner" classes are encoded as strings
    /// (you can convert such string into a proper class object by running
    /// `c = eval(class_string)`).
    ///
    /// The meaning of `oscillation_types` is the same as in `Classification.classify_phenotypes`:
    /// `Forbidden` oscillation means the attractor must be a subset of the phenotype, `allowed`
    /// oscillation means the attractor must intersect the phenotype, and `required` oscillation
    /// means that the attractor must intersect the phenotype, but can't be its subset.
    ///
    /// Similar to `Classification.classify_phenotypes`, the method allows you to provide
    /// a collection of `traps` that will be considered instead of the network attractors.
    /// The method checks that these sets are indeed trap sets, but they do not have to be
    /// minimal (i.e. attractors). If `traps` are given, the classification treats each trap
    /// set as a separate "attractor".
    ///
    /// Finally, if `count_multiplicity` is set to `False`, the method will lump together
    /// attractors that satisfy the same phenotypes, meaning the resulting `Class` object
    /// will only contain up to one instance of the same phenotype configuration. For example,
    /// if I have two attractors, each satisfying phenotype `"a"`, then by default the result
    /// is a `Class(["Class(['a'])", "Class(['a'])"])` (and this situation is different compared
    /// to the case when I only have one such attractor). If I set `count_multiplicity=False`,
    /// the result will be `Class(["Class(['a'])"])`, regardless of how many attractors actually
    /// satisfy the `"a"` phenotype.
    #[staticmethod]
    #[pyo3(signature = (graph, phenotypes, oscillation_types = None, traps = None, count_multiplicity = true))]
    pub fn classify_attractor_phenotypes(
        py: Python,
        graph: &AsynchronousGraph,
        phenotypes: HashMap<Class, VertexSet>,
        oscillation_types: Option<HashMap<Class, String>>,
        traps: Option<Vec<ColoredVertexSet>>,
        count_multiplicity: bool,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        // Initialize the attractor set.
        let traps = if let Some(traps) = traps {
            traps
        } else {
            Attractors::attractors(graph, None, None, py)?
        };

        let mut all_colors = graph.mk_empty_colors();
        for attr in &traps {
            all_colors = all_colors.union(&attr.colors());
        }

        let mut result = HashMap::new();
        result.insert(Class::new_native(Vec::new()), all_colors);

        for attr in &traps {
            let attr_classes = Self::classify_phenotypes(
                py,
                graph,
                phenotypes.clone(),
                oscillation_types.clone(),
                Some(attr.clone()),
            )?;
            for (cls, set) in attr_classes {
                let cls_str = cls.__repr__();
                let cls_py = Py::new(py, Class::new_native(vec![cls_str]))?;
                if count_multiplicity {
                    result = Self::append(result, cls_py.into_bound(py), set)?;
                } else {
                    result = Self::ensure(result, cls_py.into_bound(py), set)?;
                }
            }
        }

        Ok(result)
    }

    /// Classify the colors of an `AsynchronousGraph` according to their affinity to biological
    /// phenotypes.
    ///
    /// Each phenotype is given as an arbitrary `VertexSet`, identified by a `Class` instance.
    /// Most often, phenotypes are described through pair-wise disjoint sub-spaces (see also
    /// `AsynchronousGraph.mk_subspace_vertices`). The result of this method then associates
    /// each network color with a collection of phenotypes (i.e. `Class`) that are attainable
    /// in the corresponding fully instantiated network.
    ///
    /// By default, the network exhibits a phenotype if it can stay in the phenotype set
    /// forever (i.e. there exists at least one attractor that is fully in this set). However,
    /// we also allow other types of phenotypes based on the `PhenotypeOscillation`:
    ///
    ///  - [default] `forbidden`: There *exists* an attractor that is a *subset* of the
    ///    given phenotype `VertexSet` (oscillation is forbidden).
    ///  - `required`: There *exists* an attractor that *intersects* the phenotype set, but
    ///    is not a *proper subset* (phenotype is visited intermittently).
    ///  - `allowed`: There *exists* an attractor that *intersects* the phenotype set, but
    ///    can be also a proper subset (i.e. the network either stays in the phenotype forever,
    ///    or visits it intermittently, we don't care which one).
    ///
    /// Colors that do not match any phenotype are returned with an empty class (i.e. `Class([])`).
    ///
    /// Note that this method does not count the number of attractors, and it can assign the
    /// same attractor to multiple phenotypes (for example, a network with a fixed-point `111`
    /// exhibits both phenotype `a=11*` and `b=*11`, i.e. the network will be returned within
    /// the class `Class(['a','b'])`). This is usually not a problem for disjoint phenotypes with
    /// forbidden oscillation, just beware that for more complex phenotypes,
    /// you might need additional analysis of the relevant colors.
    ///
    /// If you do also need to map attractors to phenotypes, have a look at
    /// `Classification.classify_attractor_phenotypes`.
    ///
    /// You can (optionally) provide your own attractor states (or any other relevant
    /// set of states) as the `initial_trap` argument (the method assumes this is a trap set).
    /// However, computing phenotype classification does not require precise knowledge of
    /// attractors. Thus, it can be often much faster than exact attractor computation (especially
    /// if the number of attractors is high). However, you can use this option to restrict the
    /// input space if you want to explicitly ignore certain parts of the state space.
    ///
    /// You can also combine the results of this analysis with other classifications (e.g.
    /// attractor bifurcations, or HCTL properties) using `Classification.ensure` or
    /// `Classification.append`.
    #[staticmethod]
    #[pyo3(signature = (graph, phenotypes, oscillation_types = None, initial_trap = None))]
    pub fn classify_phenotypes(
        py: Python,
        graph: &AsynchronousGraph,
        phenotypes: HashMap<Class, VertexSet>,
        oscillation_types: Option<HashMap<Class, String>>,
        initial_trap: Option<ColoredVertexSet>,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        let mut map = HashMap::new();

        let unit = graph.mk_unit_colored_vertices();
        let initial_trap = initial_trap.unwrap_or_else(|| graph.mk_unit_colored_vertices());
        if !graph
            .as_native()
            .can_post_out(initial_trap.as_native())
            .is_empty()
        {
            return throw_runtime_error(
                "Given initial trap set is not a trap set (it can be escaped).",
            );
        }

        map.insert(Class::new_native(Vec::new()), initial_trap.colors());

        for (cls, phenotype) in phenotypes {
            let p_type = oscillation_types
                .as_ref()
                .and_then(|it| it.get(&cls))
                .map(|it| extract_phenotype_type(it.as_str()))
                .transpose()?
                .unwrap_or(PhenotypeOscillationType::Forbidden);

            let phenotype = initial_trap.intersect_vertices(&phenotype);

            let phenotype_colors = match p_type {
                PhenotypeOscillationType::Forbidden => {
                    // Oscillation is forbidden. Any attractor that contains a non-phenotype state
                    // should be disregarded.

                    // Identify all states that can reach something that is not `phenotype`,
                    // remove them, and take all colors that still appear in the set (there
                    // exists at least one attractor that is fully contained in the phenotype).
                    let not_phenotype = unit.minus(&phenotype);
                    let not_phenotype = Reachability::reach_bwd(py, graph, &not_phenotype)?;
                    let always_phenotype = phenotype.minus(&not_phenotype);
                    always_phenotype.colors()
                }
                PhenotypeOscillationType::Required => {
                    // Oscillation is required. Select all attractors that intersect the phenotype
                    // set, but are not fully contained in it.

                    // This one is a bit more tricky. The idea is that if we remove every attractor
                    // that is fully contained in the `phenotype` set as well as every attractor
                    // that is fully outside, we get a trap set with all attractors that intersect
                    // the `phenotype` set but are not contained in it.
                    let not_phenotype = unit.minus(&phenotype);
                    let not_phenotype = Reachability::reach_bwd(py, graph, &not_phenotype)?;
                    let always_phenotype = phenotype.minus(&not_phenotype);
                    let is_phenotype = unit.intersect(&phenotype);
                    let is_phenotype = Reachability::reach_bwd(py, graph, &is_phenotype)?;
                    let never_phenotype = unit.minus(&is_phenotype);
                    let can_be_never_or_always = always_phenotype.union(&never_phenotype);
                    let can_be_never_or_always =
                        Reachability::reach_bwd(py, graph, &can_be_never_or_always)?;
                    let always_mixed = unit.minus(&can_be_never_or_always);
                    always_mixed.colors()
                }
                PhenotypeOscillationType::Allowed => {
                    // Oscillation is allowed. Any attractor that is not fully *outside* the
                    // phenotype set is valid here.

                    // This is basically negating the phenotype set, and then finding all attractors
                    // that fully reside in the negated set (forbidden oscillation) and
                    // disregarding them.
                    let is_phenotype = unit.intersect(&phenotype);
                    let is_phenotype = Reachability::reach_bwd(py, graph, &is_phenotype)?;
                    let never_phenotype = unit.minus(&is_phenotype);
                    let can_reach_never_phenotype =
                        Reachability::reach_bwd(py, graph, &never_phenotype)?;
                    let allowed_phenotype = unit.minus(&can_reach_never_phenotype);
                    allowed_phenotype.colors()
                }
            };

            let py_cls = Py::new(py, cls)?;
            map = Classification::append(map, py_cls.into_bound(py), phenotype_colors)?
        }

        Ok(map)
    }

    /// Classify the behavior of the given `graph` based on the specified
    /// `HctlFormula` properties.
    ///
    /// Optionally, you can also give a collection of assertions that restrict
    /// the applicable graph colors.
    ///
    /// Note that this method internally creates a dedicated `AsynchronousGraph` with enough
    /// symbolic variables to check all the provided properties/assertions. However, the results
    /// are always transformed back into an encoding that is valid for the `graph` that is given
    /// as the first argument.
    ///
    #[staticmethod]
    #[pyo3(signature = (graph, properties, assertions = None, substitution = None))]
    pub fn classify_dynamic_properties(
        py: Python,
        graph: &AsynchronousGraph,
        properties: &Bound<'_, PyDict>,
        assertions: Option<&Bound<'_, PyList>>,
        substitution: Option<HashMap<String, ColoredVertexSet>>,
    ) -> PyResult<HashMap<Class, ColorSet>> {
        let mut max_var_count = 0;
        let mut hctl_assertions = Vec::new();
        let mut hctl_str_assertions = Vec::new();
        if let Some(assertions) = assertions {
            for it in assertions {
                let formula = HctlFormula::new(&it, true, Some(graph.symbolic_context().get()))?;
                max_var_count = max(max_var_count, formula.used_state_variables().len());
                hctl_str_assertions.push(formula.__str__());
                hctl_assertions.push(formula);
            }
        }

        let mut hctl_properties = Vec::new();
        let mut hctl_str_properties = Vec::new();
        for (k, v) in properties {
            let name = k.extract::<String>()?;
            let formula = HctlFormula::new(&v, true, Some(graph.symbolic_context().get()))?;
            max_var_count = max(max_var_count, formula.used_state_variables().len());
            hctl_str_properties.push((name.clone(), formula.__str__()));
            hctl_properties.push((name, formula));
        }

        let max_var_count = u16::try_from(max_var_count)
            .map_err(|_e| runtime_error("Too many quantified variables."))?;

        let mc_graph = if let Some(network) = graph.as_native().as_network() {
            get_extended_symbolic_graph(network, max_var_count)
        } else {
            let network = graph
                .as_native()
                .reconstruct_network()
                .ok_or_else(|| runtime_error("Cannot extract Boolean network"))?;
            get_extended_symbolic_graph(&network, max_var_count)
        }
        .map_err(runtime_error)?;

        let native_substitution = if let Some(substitution) = substitution {
            substitution
                .into_iter()
                .map(|(a, b)| (a, b.as_native().clone()))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        // First, filter assertions:

        let str_vec = hctl_str_assertions
            .iter()
            .map(|it| it.as_str())
            .collect::<Vec<_>>();
        let results =
            model_check_multiple_extended_formulae_dirty(str_vec, &mc_graph, &native_substitution)
                .map_err(runtime_error)?;

        let mut valid_colors = mc_graph.mk_unit_colors();
        for set in results {
            // We consider the "universal" interpretation of HCTL, i.e. formula holds only if it
            // holds for every state.
            let invalid_set = mc_graph.unit_colored_vertices().minus(&set);
            valid_colors = valid_colors.minus(&invalid_set.colors());
        }

        if valid_colors.is_empty() {
            // Nothing satisfies these assertions.
            return Ok(HashMap::new());
        }

        // Then, we look at the actual properties:

        let str_vec = hctl_str_properties
            .iter()
            .map(|(_, value)| value.as_str())
            .collect::<Vec<_>>();
        let results =
            model_check_multiple_extended_formulae_dirty(str_vec, &mc_graph, &native_substitution)
                .map_err(runtime_error)?;

        let valid_colors_sanitized = graph
            .as_native()
            .transfer_colors_from(&valid_colors, &mc_graph)
            .ok_or_else(|| runtime_error("Cannot sanitize color set."))?;

        let mut classification = HashMap::new();
        classification.insert(
            Class::new_native(vec![]),
            ColorSet::mk_native(graph.symbolic_context(), valid_colors_sanitized),
        );

        for ((name, _), set) in hctl_properties.into_iter().zip(results) {
            let invalid_set = mc_graph.unit_colored_vertices().minus(&set);
            let valid_set = mc_graph.unit_colors().minus(&invalid_set.colors());
            let valid_set_sanitized = graph
                .as_native()
                .transfer_colors_from(&valid_set, &mc_graph)
                .ok_or_else(|| runtime_error("Cannot sanitize color set."))?;

            let cls = Class::new_native(vec![name]);

            classification = Classification::append(
                classification,
                Py::new(py, cls)?.into_bound(py),
                ColorSet::mk_native(graph.symbolic_context(), valid_set_sanitized),
            )?;
        }

        Ok(classification)
    }
}

/// Extract an annotation object from a Python object.
///
/// An annotation object can be either:
///  - A single line referencing a file path.
///  - Multiple lines representing the contents of an annotated file.
///  - The [PyModelAnnotation] itself.
fn extract_annotations(py: Python, annotations: &Bound<'_, PyAny>) -> PyResult<ModelAnnotation> {
    if let Ok(string) = annotations.extract::<String>() {
        if string.contains('\n') {
            // This is a model string.
            ModelAnnotation::from_aeon(py, string.as_str())
        } else {
            // This is a model path.
            match std::fs::read_to_string(string.as_str()) {
                Ok(contents) => ModelAnnotation::from_aeon(py, contents.as_str()),
                Err(e) => throw_runtime_error(format!("Cannot read path `{string}`: {e:?}.")),
            }
        }
    } else if let Ok(annotations) = annotations.extract::<ModelAnnotation>() {
        Ok(annotations)
    } else {
        throw_type_error("Expected annotation object, model string or path.")
    }
}

fn encode_behavior(x: Behaviour) -> String {
    match x {
        Behaviour::Stability => "stability",
        Behaviour::Oscillation => "oscillation",
        Behaviour::Disorder => "disorder",
    }
    .to_string()
}
