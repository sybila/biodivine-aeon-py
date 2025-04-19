use pyo3::pyclass;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// A representation for model data provided by the Biodivine Boolean Models
/// database endpoint. It is used to deserialize the model data from the JSON
/// response.
///
/// The `modelData` field is a string representation of the Boolean network
/// in aeon format. The rest of the fields are metadata about the model.
#[pyclass(module = "biodivine_aeon")]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BbmModel {
    pub id: String,
    pub name: String,
    pub url_publication: Option<String>,
    pub url_model: Option<String>,
    pub keywords: Vec<String>,
    pub variables: u32,
    pub inputs: u32,
    pub regulations: u32,
    pub notes: Option<String>,
    pub bib: Option<String>,
    #[serde(rename = "modelData", deserialize_with = "deserialize_model_data")]
    pub model_data: String, // Deserialize directly into a String
}

/// Custom deserialization function for the `modelData` field.
/// Converts the raw byte data from the JSON response into a UTF-8 string.
fn deserialize_model_data<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct RawModelData {
        #[serde(rename = "type")]
        _data_type: String,
        data: Vec<u8>,
    }

    let raw: RawModelData = RawModelData::deserialize(deserializer)?;
    String::from_utf8(raw.data)
        .map_err(|e| de::Error::custom(format!("Failed to convert model data to string: {}", e)))
}
