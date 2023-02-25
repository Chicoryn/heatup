use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub struct Note {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub output: String,
}
