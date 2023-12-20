//! Loading of various input components of the model, mainly of various properties/assertions.

use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};

use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

/// Read the list of assertions from an `.aeon` model annotation object.
///
/// The assertions are expected to appear as `#!dynamic_assertion: FORMULA` model annotations
/// and they are returned in declaration order.
pub fn read_model_assertions(annotations: &ModelAnnotation) -> Vec<String> {
    let Some(list) = annotations.get_value(&["dynamic_assertion"]) else {
        return Vec::new();
    };
    list.lines().map(|it| it.to_string()).collect()
}

/// Read the list of named properties from an `.aeon` model annotation object.
///
/// The properties are expected to appear as `#!dynamic_property: NAME: FORMULA` model annotations.
/// They are returned in alphabetic order w.r.t. the property name.
pub fn read_model_properties(
    annotations: &ModelAnnotation,
) -> Result<Vec<(String, String)>, String> {
    let Some(property_node) = annotations.get_child(&["dynamic_property"]) else {
        return Ok(Vec::new());
    };
    let mut properties = Vec::with_capacity(property_node.children().len());
    for (name, child) in property_node.children() {
        if !child.children().is_empty() {
            // TODO:
            //  This might actually be a valid (if ugly) way for adding extra meta-data to
            //  properties, but let's forbid it for now and we can enable it later if
            //  there is an actual use for it.
            return Err(format!("Property `{name}` contains nested values."));
        }
        let Some(value) = child.value() else {
            return Err(format!("Found empty dynamic property `{name}`."));
        };
        if value.lines().count() > 1 {
            return Err(format!("Found multiple properties named `{name}`."));
        }
        properties.push((name.clone(), value.clone()));
    }
    // Sort alphabetically to avoid possible non-determinism down the line.
    properties.sort_by(|(x, _), (y, _)| x.cmp(y));
    Ok(properties)
}

/// Combine all HCTL assertions in the given list into a single conjunction of assertions.
pub fn build_combined_assertion(assertions: &[String]) -> String {
    if assertions.is_empty() {
        "true".to_string()
    } else {
        // Add parenthesis to each assertion.
        let assertions: Vec<String> = assertions.iter().map(|it| format!("({it})")).collect();
        // Join them into one big conjunction.
        assertions.join(" & ")
    }
}

/// Read the contents of a file from a zip archive into a string.
fn read_zip_file(reader: &mut ZipArchive<File>, file_name: &str) -> String {
    let mut contents = String::new();
    let mut file = reader.by_name(file_name).unwrap();
    file.read_to_string(&mut contents).unwrap();
    contents
}

/// Load the archive containing results produced by the classifier.
/// This function can also be used to load any `classification archives` of the same format (e.g.,
/// those produced by the `build_classification_archive` function).
///
/// Return mapping `category name -> color set` and whole model string in aeon format.
/// Category names are simply taken from BDD-file names in the archive.
pub fn load_classification_archive(
    archive_path: String,
) -> Result<(HashMap<String, GraphColors>, String), String> {
    // Open the zip archive with classification results.
    let archive_file = File::open(archive_path).map_err(|e| format!("{e:?}"))?;
    let mut archive = ZipArchive::new(archive_file).map_err(|e| format!("{e:?}"))?;

    // Load the BN model (from the archive) and generate the extended STG.
    let aeon_str = read_zip_file(&mut archive, "model.aeon");
    let bn = BooleanNetwork::try_from(aeon_str.as_str())?;
    let graph = SymbolicAsyncGraph::new(&bn)?;

    // collect the classification outcomes (colored sets) from the individual BDD dumps
    let mut categories = HashMap::new();

    // Load all class BDDs from files in the archive.
    let files = archive
        .file_names()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    for file in files {
        if !file.starts_with("bdd_dump_") {
            // Only read BDD dumps.
            continue;
        }

        let bdd_string = read_zip_file(&mut archive, file.as_str());
        let bdd = Bdd::from_string(bdd_string.as_str());
        let color_set = GraphColors::new(bdd, graph.symbolic_context());

        let category_id = file.strip_prefix("bdd_dump_").unwrap();
        let category_id = category_id.strip_suffix(".txt").unwrap();

        // The insert should create a new item, otherwise the archive is malformed.
        assert!(categories
            .insert(category_id.to_string(), color_set)
            .is_none());
    }

    Ok((categories, aeon_str))
}
