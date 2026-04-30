use serde::{Serialize, Deserialize};

// enum
#[derive(Serialize, Deserialize)]
pub enum FileNode {
    File(File),
    Directory(Directory),
}
// end

// structs
#[derive(Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Directory {
    pub name: String,
    pub children: Vec<FileNode>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitKey {
    pub token: String,
    pub project: Option<String>,
}
// end
