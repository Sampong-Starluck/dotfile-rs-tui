use nanoserde::{DeJson, SerJson};

#[derive(Clone, PartialEq, DeJson, SerJson, Debug)]
pub struct ShellEntry {
    pub id: String,
    pub name: String,
    #[nserde(default)]
    pub hidden: bool,
    #[nserde(default)]
    pub description: String,
    #[nserde(default)]
    pub order: i32,
    #[nserde(default)]
    pub requires: Vec<String>,
}

#[derive(DeJson)]
pub struct ShellsFile {
    pub shells: Vec<ShellEntry>,
}