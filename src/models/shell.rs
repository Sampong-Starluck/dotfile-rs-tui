use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ShellConfig {
    pub shells: Vec<ShellEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShellEntry {
    pub id: String,
    pub name: String,
    pub hidden: bool,
    pub description: String,
    pub order: u32,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}