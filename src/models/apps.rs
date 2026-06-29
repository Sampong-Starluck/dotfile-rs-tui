use std::collections::HashMap;
use nanoserde::{DeJson, SerJson};

pub type Apps = Vec<AppSection>;

#[derive(Clone, PartialEq, DeJson, SerJson, Debug)]
pub struct AppSection {
    pub section: String,
    pub apps: Vec<AppEntry>,
}

#[derive(Clone, PartialEq, DeJson, SerJson, Debug)]
pub struct AppEntry {
    pub name: String,
    pub id: String,
    pub platforms: HashMap<String, HashMap<String, String>>,
}