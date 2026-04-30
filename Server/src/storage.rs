use crate::models::{FileNode, File, Directory, GitKey};
use crate::config::GIT_KEYS_DIR;
use std::{
    fs,
    path::Path,
};

pub fn git_keys_file() -> String {
    format!("{}/git_keys.json", GIT_KEYS_DIR)
}

pub fn get_all_keys() -> Vec<GitKey> {
    let keys_file = git_keys_file();

    let raw = fs::read_to_string(keys_file).expect("couldn't read file"); 
    let keys: Vec<GitKey> = serde_json::from_str(&raw).expect("");
    
    return keys;
}

pub fn folder_rec(path: &Path) -> FileNode {
    let name = path.file_name().unwrap().to_string_lossy().to_string();

    if path.is_dir() {
        // go deeper if this path is a folder
        let mut children = Vec::new();

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let child = folder_rec(&entry.path());
            
            children.push(child);
        }

        FileNode::Directory(Directory { name, children })
    
    } else {
        // read if this path is a file
        let content = fs::read_to_string(path)
            .unwrap_or_default();
        FileNode::File(File { name, content })
    }
}
