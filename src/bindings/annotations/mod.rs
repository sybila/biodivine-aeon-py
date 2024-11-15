use super::lib_param_bn::model_annotation::ModelAnnotation;
use pyo3::prelude::*;

/// An extension of `ModelAnnotation` that provides type-safe access to all annotation data
/// that is officially supported by AEON as annotations of a `BooleanNetwork` or
/// a `RegulatoryGraph`.
///
/// Note that you can still access any "raw" annotation data the same way you would on
/// any instance of `ModelAnnotation`.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct NetworkAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct VariableAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct VariableIdentifiers();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct RegulationAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct RegulationIdentifiers();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct ParameterAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
struct ParameterIdentifier();
