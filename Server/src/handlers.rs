use base64::{Engine, engine::general_purpose::STANDARD};

use crate::{
    config::{REPOS_DIR, compare_passwd}, 
    models::GitKey, response::{send_back, send_back_key, send_back_keys, send_back_repo}, storage::{folder_rec, get_all_keys, git_keys_file},
};

use std::{
    io::{BufReader, BufWriter, prelude::*},
    collections::HashMap,      
    net::TcpStream,
    error::Error,
    fs,
    path::Path,
    io::Write,
};

use path_security::validate_path;

pub fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buff_reader = BufReader::new(&stream);
    let mut request_line = String::new();
    buff_reader.read_line(&mut request_line)?;
    let request_line = request_line.trim_end();        

    let method = request_line
        .split_whitespace()
        .next()
        .unwrap_or("");

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");
    let path_without_query = path
        .splitn(2, '?')
        .nth(0)
        .unwrap_or("");
    let query = path
        .splitn(2, '?')
        .nth(1)
        .unwrap_or("");
    let params: Vec<(&str, &str)> = query
        .split('&')
        .filter_map(|pair| pair.split_once("="))
        .collect();
    

    // before we do anything else we should check if the path we got doesnt contain ../.. or something like that
    // we only want the user to be able to access the repo folder and nothing else
    let relative_path = path_without_query.trim_start_matches('/');
    let user_path = Path::new(relative_path);
    let base_path = Path::new(REPOS_DIR);
    let safe_path = match validate_path(user_path, base_path) {
        Ok(safe_path) => safe_path,
        Err(_) => {
            let message = "Path blocked because path traversal was detected";
            send_back(message, &stream, 403);
            return Ok(());
        }
    };
    // here we convert our PathBuffer to a &str because we expect a &str everywhere else
    // and we use anyhow because rust converts any error you have into anyhow::Error when you use ?
    let new_path = safe_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains invalid UTF-8"))?;


    // read the headers line by line until we get to a blank line
    let mut body_length = 0;
    let mut headers = HashMap::new();

    loop {
        let mut line = String::new();
        buff_reader.read_line(&mut line)?;
        let line = line.trim_end();

        if line.is_empty() {
            break; // blank line -> end of header
        }

        if let Some(val) = line.to_lowercase().strip_prefix("content-length:") {
            body_length = val.trim().parse().unwrap_or(0);
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_lowercase(), value.to_string());
        }
    }

    // get the password now
    let passwd = headers.get("pwd").map(|s| s.as_str()).unwrap_or("");

    // read exact body length bytes for the body
    let mut body_bytes = vec![0u8; body_length];
    if body_length > 0 {
        buff_reader.read_exact(&mut body_bytes)?;
    }

    // finally the actual body
    let body = String::from_utf8_lossy(&body_bytes);

    if method == "GET" {
        // Getting a repo
        let segments: Vec<&str> = path
            .splitn(3, '/')
            .collect();

        if segments.get(1) == Some(&"pull") {
            let repo_name = segments
                .get(2)
                .unwrap_or(&"");

            if let Err(_e) = handle_pull_repo(repo_name, &stream) {
                let message = "Failed to pull the requested repo";
                send_back(message, &stream, 500);
                return Ok(());
            }
        } else if segments.get(1) == Some(&"keys") {
            if let Err(_e) = compare_passwd(passwd, &stream) {
                return Ok(());
            }

            let key_name = segments
                .get(2)
                .unwrap_or(&"");

            if let Err(_e) = handle_pull_git_keys(key_name, &stream) {
                let message = "Failed to get the requested key";
                send_back(message, &stream, 500);
                return Ok(());
            }
        } else if segments.get(1) == Some(&"getkeys") {
            if let Err(_e) = compare_passwd(passwd, &stream) {
                return Ok(());
            } 

            if let Err(_e) = handle_get_all_keys(&stream) {
                let message = "Failed to get all keys";
                send_back(message, &stream, 500);
                return Ok(());
            }
        } else if segments.get(1) == Some(&"getrepos") {
            if let Err(_e) = handle_get_all_repos(&stream) {
                let message = "Failed to get all repo names";
                send_back(message, &stream, 500);
                return Ok(());
            }
        }
    } else if method == "PUT" {
        if let Err(_e) = compare_passwd(passwd, &stream) {
            return Ok(());
        }

        // Pushing a file to a specific repo 
        let segments: Vec<&str> = new_path
            .splitn(3, '/')
            .collect();

        if segments.get(1) == Some(&"update") {
            let file_name = segments
                .get(2)
                .unwrap_or(&"");
            
            if let Err(_e) = handle_update_file(body.as_ref(), file_name, &stream, params) {
                let message = "Failed to write to file";
                send_back(message, &stream, 500);
                return Ok(());
            }
        } else if segments.get(1) == Some(&"key") {
            let key_name = segments
                .get(2)
                .unwrap_or(&"");

            if let Err(_e) = handle_add_key(body.as_ref(), key_name, &stream) {
                let message = "Failed to add key to list";
                send_back(message, &stream, 500);
                return Ok(());
            }
        }
    } else if method == "POST" {
        if let Err(_e) = compare_passwd(passwd, &stream) {
            return Ok(());
        } 

        // Making a new dir/repo
        if new_path == "/repo/new" {
           if let Err(_e) = handle_create_dir(params, &stream) {
                let message = "Failed to create dir/repo";
                send_back(message, &stream, 500);
                return Ok(());
           }
        } 
    }

    Ok(())
}


pub fn handle_get_all_repos(stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut folder_names = String::new();
    let entries = fs::read_dir(REPOS_DIR)?;

    for entry in entries {
        let entry = entry.unwrap();
        let data = entry.metadata()?;

        if data.is_dir() {
            if let Ok(name) = entry.file_name().into_string() {
                folder_names.push_str(&name);
                folder_names.push('\n');
            }
        }
    }

    send_back(&folder_names, stream, 200);
    return Ok(());
}

pub fn handle_add_key(key: &str, key_name: &str, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let message;
    let keys_file = fs::File::create(&git_keys_file())?;

    let mut keys: Vec<GitKey> = get_all_keys();
    let mut already_key: bool = false;
    let mut writer = BufWriter::new(keys_file);

    for git_key in &keys {
        if git_key.project == Some(key.to_string()) || git_key.token == key_name {
            already_key = true;
        }
    }

    if already_key == true {
        message = "Key already exists";
        send_back(message, &stream, 400);
        return Ok(());
    }

    let new_key: GitKey = GitKey { token: key_name.to_string(), project: Some(key.to_string()) };

    keys.push(new_key);

    serde_json::to_writer(&mut writer, &keys)?;
    writer.flush()?;

    return Ok(());
}

pub fn handle_get_all_keys(stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut message: String = String::new();

    let keys: Vec<GitKey> = get_all_keys();

    for key in keys {
        message += &(key.project.as_deref().unwrap_or("").to_owned() + "\n");
    }

    send_back_keys(stream, 200, &message);
    return Ok(());
}

pub fn handle_pull_git_keys(key_name: &str, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let message;

    if key_name == "" {
        eprintln!("[pull] missing key name");
        message = "Couldnt get the key name";
        send_back(message, stream, 404);
        return Ok(());
    }

    let keys_file = git_keys_file();

    let raw = std::fs::read_to_string(keys_file).expect("couldn't read file"); 
    let keys: Vec<GitKey> = serde_json::from_str(&raw).expect("");

    for key in &keys {
        if key.project == Some((&key_name).to_string()) {
            message = &key.token;
            send_back_key(stream, 200, message);
            return Ok(());
        }
    }

    message = "Couldnt find git key with that name";
    send_back(message, stream, 404);
    return Ok(());
}

pub fn handle_pull_repo(repo_name: &str, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let message;

    if repo_name == "" {
        eprintln!("[pull] missing repo name");
        message = "Couldnt get the repo name";
        send_back(message, stream, 404);
        return Ok(());
    }   

    if !(Path::new(&(REPOS_DIR.to_owned() + "/" + repo_name)).exists()) {
        eprintln!("[pull] repo doesn't exist: {repo_name}");
        message = "Dir/Repo with that name doesnt exist";  
        send_back(message, stream, 404);
        return Ok(());
    } 

    // we need to go through the entire folder recursively
    let folder = folder_rec(Path::new(&format!("{REPOS_DIR}/{repo_name}")));

    send_back_repo(stream, 200, folder);
    return Ok(());
}



pub fn handle_update_file(file_contents: &str, file_name: &str, stream: &TcpStream, params: Vec<(&str, &str)>) -> Result<(), Box<dyn Error>> {
   let message;

    if file_name == "" {
        eprintln!("[update] missing file name");
        message = "Couldnt get a file name (might be a server error)";
        send_back(message, stream, 404);
        return Ok(());
    }

    if params.is_empty() {
        eprintln!("[update] no params");
        message = "Couldnt get the repo/dir name to which to push to";
        send_back(message, stream, 404);
        return Ok(());
    }


    let (name_key, name_value) = params.get(0).unwrap();

    if *name_key != "where" {
        eprintln!("[update] wrong param key: {name_key}");
        message = "Couldnt get the repo/dir name to which to push to";
        send_back(message, stream, 404);
        return Ok(());
    }

    let repo_path = format!("{REPOS_DIR}/{name_value}");

    if !(Path::new(&(REPOS_DIR.to_owned() + "/" + name_value)).exists()) {
        eprintln!("[update] repo doesn't exist: {repo_path}");
        message = "Dir/Repo with that name doesnt exist, create it before pushing";  
        send_back(message, stream, 404);
        return Ok(());
    } 

    let decoded = STANDARD.decode(file_contents.trim()).map_err(|e| {
        eprintln!("[update] base64 decode failed: {e}");
        e
    })?;

    
    let file_path = &(REPOS_DIR.to_owned() + "/" + name_value + "/" + file_name);
    // if no check failed then we update/create the file 
    if Path::new(file_path).exists() {
        // if file exists update
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;

        file.write_all(&decoded)?;

        file.flush()?;

        message = "Sucessfully updated existing file";
        send_back(message, stream, 200);
        return Ok(());
    } else {
        // if file doesnt exist create it
        let mut file = fs::File::create(file_path)?;

        file.write_all(&decoded)?;

        file.flush()?;

        message = "Sucessfully created new file and wrote to it";
        send_back(message, stream, 201);
        return Ok(());
    }
}

pub fn handle_create_dir(params: Vec<(&str, &str)>, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let message;

    if params.is_empty() {
        message = "Couldnt get the name the new dir/repo"; 
        send_back(message, stream, 404);
        return Ok(());
    } 


    let (name_key, name_value) = params.get(0).unwrap();

    if *name_key == "name" {
        println!("Got a name");
        // check if the actual name is just empty
        if *name_value == "" {
            message = "No dir/repo name given";
            send_back(message, stream, 404);
            return Ok(());
        }

        // check if dir already exists
        if Path::new(&(REPOS_DIR.to_owned() + "/" + name_value)).exists() {
            message = "Dir/Repo with the same name already exists";
            send_back(message, stream, 404);
            return Ok(());
        }

        // after checking if everything is valid we cna create it
        match fs::create_dir(&(REPOS_DIR.to_owned() + "/" + name_value)) {
            Ok(()) => {
                message = "Succesfully created new dir/repo";
                send_back(message, stream, 201);
                return Ok(());
            }
            Err(e) => {
               println!("Error when creating new dir/repo: {e}") ;

               message = "Internal Server Error";
               send_back(message, stream, 500);
               return Ok(());
            }
        }
    } else {
        // send back 404 instantly
        message = "Couldnt get the name the new dir/repo"; 
        send_back(message, stream, 404);
        return Ok(());
    }
}

