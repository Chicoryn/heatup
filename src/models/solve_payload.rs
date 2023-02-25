use serde::{Serialize, Deserialize};
use super::Cooldown;

#[derive(Deserialize, Serialize)]
pub struct SolvePayload {
    pub template: String,
    pub cooldowns: Vec<Cooldown>,
}
