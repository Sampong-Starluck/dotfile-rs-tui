use nanoserde::DeJson;

#[derive(Debug, Clone, DeJson)]
pub struct ShellConfig {
    pub shells: Vec<ShellEntry>,
}

#[derive(Debug, Clone, DeJson)]
pub struct ShellEntry {
    pub id: String,
    pub name: String,
    pub hidden: bool,
    pub description: String,
    pub order: u32,
    #[nserde(default)]
    pub platforms: Vec<String>,
    #[nserde(default)]
    pub requires: Vec<String>,
}