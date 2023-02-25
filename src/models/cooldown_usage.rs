#[derive(Debug)]
pub struct CooldownUsage {
    pub uid: u64,
    pub at: u64,
    pub value: f64,
    pub group_names: Vec<String>,
}