//! Finish the classification process and generate the results (report and BDD representation).

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use zip::write::{SimpleFileOptions, ZipWriter};

/// Create classification archive for an arbitrary "map" of `string -> color set`.
pub fn build_classification_archive(
    categories: HashMap<String, GraphColors>,
    archive_name: &str,
    original_model_str: &str,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    // If there are some non-existing dirs in the path, create them.
    let prefix = archive_path
        .parent()
        .ok_or(std::io::Error::other("Invalid path."))?;
    std::fs::create_dir_all(prefix)?;

    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    for (category_name, category_colors) in categories.iter() {
        if !category_colors.is_empty() {
            // If the BDD is not empty, the results go directly into the zip archive.
            let bdd_file_name = format!("bdd_dump_{category_name}.txt");
            zip_writer
                .start_file(bdd_file_name.as_str(), SimpleFileOptions::default())
                .map_err(std::io::Error::from)?;

            category_colors.as_bdd().write_as_string(&mut zip_writer)?;
        }
    }

    // Include the original model in the result bundle (we need to load it later).
    zip_writer
        .start_file("model.aeon", SimpleFileOptions::default())
        .map_err(std::io::Error::from)?;
    write!(zip_writer, "{original_model_str}")?;

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}
