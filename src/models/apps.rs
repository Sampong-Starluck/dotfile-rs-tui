use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type Apps = Vec<AppSection>;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct AppSection {
    pub section: String,
    pub apps: Vec<AppEntry>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct AppEntry {
    pub name: String,
    pub id: String,
    pub platforms: HashMap<String, HashMap<String, String>>,
}