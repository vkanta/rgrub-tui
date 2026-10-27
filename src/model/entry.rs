use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrubCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrubEntry {
    pub name: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub commands: Vec<GrubCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrubMenu {
    Entry(GrubEntry),
    Submenu {
        name: String,
        id: Option<String>,
        entries: Vec<GrubMenu>,
    },
}
