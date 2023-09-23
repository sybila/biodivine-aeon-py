//! Finish the classification process and generate the results (report and BDD representation).

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use std::collections::HashMap;

use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;

use zip::write::{FileOptions, ZipWriter};

/// Transform integer into a corresponding binary number of the given length.
///
/// If the integer "bit width" is larger than the given length, it is truncated. If it is smaller,
/// the result is padded with zeroes to ensure `result.len() == bits_num`.
///
/// The result is given in MSB first (most significant bit first) format (as opposed to LSB, which
/// is a bit more common in other applications). This means that when the vector is printed (with
/// first element being the left-most printed item), it can be read left-to-right as the binary
/// representation of the input `number`.
fn int_to_bool_vec(number: i32, bits_num: usize) -> Vec<bool> {
    let mut bits = vec![false; bits_num]; // Pre-allocate the values in one operation.
    for i in 0..bits_num {
        let msb_index = bits_num - i - 1; // Invert index to ensure MSB bit order.
        bits[msb_index] = ((number >> i) & 1) == 1;
    }
    bits
}

/// Convert a vector of bools to the corresponding binary string.
fn bool_vec_to_string(bool_data: &[bool]) -> String {
    bool_data
        .iter()
        .map(|x| if *x { '1' } else { '0' })
        .collect()
}

/// Prepare the initial part of the report regarding results for assertion formulae and
/// results for individual property formulae.
fn prepare_report_intro(
    assertion_formulae: &[String],
    all_valid_colors: &GraphColors,
    named_property_formulae: &[(String, String)],
    property_results: &[GraphColors],
) -> Result<Vec<u8>, std::io::Error> {
    // We will first write the report into an intermediate buffer,
    // because we want to write it into the zip archive at the end
    // once all results are computed.
    let mut report = Vec::new();

    // Write the list of assertions.
    writeln!(report, "### Assertion formulae")?;
    writeln!(report)?;
    for assertion in assertion_formulae {
        writeln!(report, "# {assertion}")?;
    }
    writeln!(
        report,
        "{:.0} colors satisfy all assertions",
        all_valid_colors.approx_cardinality()
    )?;
    writeln!(report)?;

    // Write results for each property.
    writeln!(report, "### Property formulae individually")?;
    writeln!(report)?;
    for i in 0..named_property_formulae.len() {
        let (name, property) = &named_property_formulae[i];
        writeln!(report, "# {name}  |  {property}")?;
        let cardinality = property_results[i].approx_cardinality();
        writeln!(report, "{cardinality:.0} colors satisfy this property")?;
        writeln!(report)?;
    }

    // Output info regarding the classification.
    writeln!(report, "### Classes")?;
    writeln!(report)?;

    Ok(report)
}

/// Write a short summary regarding each category of the color decomposition, and dump a BDD
/// encoding the colors, all into the `archive_name` zip.
///
///  - `assertion_formulae`: list of assertion formulae
///  - `all_valid_colors`: represents a "unit color set", i.e. all colors satisfying the
///     assertion formulae.
///  - `named_property_formulae`: lists the property names with their HCTL formula strings.
///  - `property_results`: lists the symbolic color set results for each property.
///  - `archive_name`: name of the `.zip` archive with results.
///  - `original_model_str`: original model in the aeon format
///
/// Each result category is given by a set of colors that satisfy exactly the same properties.
pub fn write_classifier_output(
    assertion_formulae: &[String],
    all_valid_colors: &GraphColors,
    named_property_formulae: &[(String, String)],
    property_results: &[GraphColors],
    archive_name: &str,
    original_model_str: &str,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path
        .parent()
        .ok_or(std::io::Error::new(ErrorKind::Other, "Invalid path."))?;
    std::fs::create_dir_all(prefix)?;

    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    // We will first write the report into an intermediate buffer,
    // because we want to write it into the zip archive at the end
    // once all results are computed.
    let mut report = prepare_report_intro(
        assertion_formulae,
        all_valid_colors,
        named_property_formulae,
        property_results,
    )?;

    // If this is broken, the number of properties is too high
    // to enumerate the combinations explicitly.
    assert!(property_results.len() < 31);
    let number_of_combinations = 1 << property_results.len();

    for i in 0..number_of_combinations {
        let validity = int_to_bool_vec(i, property_results.len());

        // Build the color set of this category based on the validity vector for this index.
        let mut category_colors = all_valid_colors.clone();
        for (set, is_valid) in property_results.iter().zip(validity.iter()) {
            if *is_valid {
                category_colors = category_colors.intersect(set);
            } else {
                category_colors = category_colors.minus(set);
            }
        }

        writeln!(report, "# {}", bool_vec_to_string(&validity))?;
        writeln!(
            report,
            "{:.0} colors in this category",
            category_colors.approx_cardinality()
        )?;
        writeln!(report)?;

        if !category_colors.is_empty() {
            // If the BDD is not empty, the results go directly into the zip archive.
            let bdd_file_name = format!("bdd_dump_{}.txt", bool_vec_to_string(&validity));
            zip_writer
                .start_file(&bdd_file_name, FileOptions::default())
                .map_err(std::io::Error::from)?;

            category_colors.as_bdd().write_as_string(&mut zip_writer)?;
        }
    }

    // Finally, we can write the report.
    zip_writer
        .start_file("report.txt", FileOptions::default())
        .map_err(std::io::Error::from)?;
    zip_writer.write_all(&report)?;

    // Include the original model in the result bundle (we need to load it later).
    zip_writer
        .start_file("model.aeon", FileOptions::default())
        .map_err(std::io::Error::from)?;
    write!(zip_writer, "{original_model_str}")?;

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}

/// Create classification archive for an arbitrary "map" of `string -> color set`.
pub fn build_classification_archive(
    categories: HashMap<String, GraphColors>,
    archive_name: &str,
    original_model_str: &str,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path
        .parent()
        .ok_or(std::io::Error::new(ErrorKind::Other, "Invalid path."))?;
    std::fs::create_dir_all(prefix)?;

    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    for (category_name, category_colors) in categories.iter() {
        if !category_colors.is_empty() {
            // If the BDD is not empty, the results go directly into the zip archive.
            let bdd_file_name = format!("bdd_dump_{}.txt", category_name);
            zip_writer
                .start_file(&bdd_file_name, FileOptions::default())
                .map_err(std::io::Error::from)?;

            category_colors.as_bdd().write_as_string(&mut zip_writer)?;
        }
    }

    // Include the original model in the result bundle (we need to load it later).
    zip_writer
        .start_file("model.aeon", FileOptions::default())
        .map_err(std::io::Error::from)?;
    write!(zip_writer, "{original_model_str}")?;

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}

/// Write a short summary regarding the classification computation where the assertions were
/// not satisfied.
pub fn write_empty_report(
    assertion_formulae: &[String],
    archive_name: &str,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    // Here, we can write the empty report directly because there is nothing else to compute.
    zip_writer
        .start_file("report.txt", FileOptions::default())
        .map_err(std::io::Error::from)?;

    writeln!(zip_writer, "### Assertion formulae")?;
    writeln!(zip_writer)?;
    for assertion in assertion_formulae {
        writeln!(zip_writer, "# {assertion}")?;
    }
    writeln!(zip_writer, "0 colors satisfy combination of all assertions")?;
    writeln!(zip_writer)?;

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::internal::classification::write_output::{bool_vec_to_string, int_to_bool_vec};

    #[test]
    fn test_int_to_bool_vec() {
        let expected_vec = vec![false, false, false];
        assert_eq!(int_to_bool_vec(0, 3), expected_vec);

        let expected_vec = vec![false, true];
        assert_eq!(int_to_bool_vec(1, 2), expected_vec);

        let expected_vec = vec![false, false, false, true];
        assert_eq!(int_to_bool_vec(1, 4), expected_vec);

        let expected_vec = vec![false, false, true, false];
        assert_eq!(int_to_bool_vec(2, 4), expected_vec);

        let expected_vec = vec![true, true, true, true];
        assert_eq!(int_to_bool_vec(15, 4), expected_vec);
    }

    #[test]
    fn test_bool_vec_to_string() {
        assert_eq!(bool_vec_to_string(&[]), "".to_string());
        assert_eq!(bool_vec_to_string(&[true, false]), "10".to_string());
        assert_eq!(bool_vec_to_string(&[true, true, false]), "110".to_string());
    }
}
