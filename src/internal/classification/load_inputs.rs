//! Loading of various model input components, mainly of various properties/assertions.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use zip::ZipArchive;

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
        assert!(
            categories
                .insert(category_id.to_string(), color_set)
                .is_none()
        );
    }

    Ok((categories, aeon_str))
}
