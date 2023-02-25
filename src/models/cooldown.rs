use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Cooldown {
    #[serde(alias = "displayName")]
    pub display_name: String,

    #[serde(alias = "cooldown")]
    pub cooldown: f64,

    #[serde(alias = "value")]
    pub value: f64,

    #[serde(alias = "groupNames")]
    pub group_names: Vec<String>,
}