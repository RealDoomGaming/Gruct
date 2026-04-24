use std::{
    // BufReader and prelude -> traits and types which let us read and write to stream
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    error::{Error},
    path::{Path},
    fs,
};
use base64::{Engine, engine::general_purpose::STANDARD};

// constants
const REPOS_DIR: &str = "/var/lib/gruct-repos";
const _LOGS_DIR: &str = "/var/log/gruct-logs";
// end

// enum
enum FileNode {
    File(File),
    Directory(Directory),
}
// end

// structs
struct File {
    name: String,
    content: String,
}

struct Directory {
    name: String,
    children: Vec<FileNode>,
}
// end

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    if !(Path::new(REPOS_DIR).exists()) {
        match fs::create_dir(REPOS_DIR) {
            Ok(()) => {}
            Err(_e) => {
                panic!("Failed to create the repos folder when starting the server for the fist time, 
                    if this problem persists either create the folder yourself or run this with sudo");
            }
        }
    } 
    // do the same with logs later

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        match handle_connection(stream) {
            Ok(_resp) => {
            }
            Err(_e) => {
            }
        }
    }    
}

fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
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

    // read the headers line by line until we get to a blank line
    let mut body_length = 0;

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
    }

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
                send_back(message, &stream, 404);
                return Ok(());
            }
        }
    } else if method == "PUT" {
        // Pushing a file to a specific repo 
        let segments: Vec<&str> = path_without_query
            .splitn(3, '/')
            .collect();

        if segments.get(1) == Some(&"update") {
            let file_name = segments
                .get(2)
                .unwrap_or(&"");
            
            if let Err(_e) = handle_update_file(body.as_ref(), file_name, &stream, params) {
                let message = "Failed to write to file";
                send_back(message, &stream, 404);
                return Ok(());
            }
        }
    } else if method == "POST" {
        // Making a new dir/repo
        if path_without_query == "/repo/new" {
           if let Err(_e) = handle_create_dir(params, &stream) {
                let message = "Failed to create dir/repo";
                send_back(message, &stream, 404);
                return Ok(());
           }
        }
    }

    /*
    let status_line = "HTTP/1.1 200 OK";
    let content = "Hellos :D";
    let con_length = content.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {con_length}\r\n\r\n{content}");
    
    stream.write_all(response.as_bytes())?;
    */

    Ok(())
}

fn handle_pull_repo(repo_name: &str, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut message = "";

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
    let folder = folder_rec(Path::new(repo_name));

    message = "Sucessfully pulled the repo/dir";
    send_back(message, stream, 200);
    return Ok(());
}

fn folder_rec(path: &Path) -> FileNode {
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


fn handle_update_file(file_contents: &str, file_name: &str, stream: &TcpStream, params: Vec<(&str, &str)>) -> Result<(), Box<dyn Error>> {
   let mut message = "";

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

fn handle_create_dir(params: Vec<(&str, &str)>, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut message = "";

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

fn send_back(message: &str, mut stream: &TcpStream, status_code: i32) {
    let message_len = message.len();

    let status_text = match status_code {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let resp = format!("HTTP/1.1 {status_code} {status_text}\r\nContent-Length: {message_len}\r\n\r\n{message}");

    stream.write_all(resp.as_bytes()).expect("Failed to Write to client");
}
