use std::{
    // BufReader and prelude -> traits and types which let us read and write to stream
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    error::Error,
    path::{PathBuf, Path},
    fs,
};

// constants
const REPOS_DIR: &str = "/var/lib/gruct-repos";
const _LOGS_DIR: &str = "/var/log/gruct-logs";
// end

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    if !(Path::new(REPOS_DIR).exists()) {
        match fs::create_dir(REPOS_DIR) {
            Ok(()) => {}
            Err(e) => {
                Err::<(), &str>("Failed to create repos dir when starting up for the first time,
                    maybe try starting with sudo or create it yourself\n Actuall error: {e}");
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

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // stream is mutable -> we can change it
    let buff_reader = BufReader::new(&stream);
    let request_line = buff_reader
        .lines()
        .next()
        .unwrap()
        .unwrap(); 

    let method = request_line
        .split_whitespace()
        .next()
        .unwrap_or("");

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap();
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

    let body = request_line
        .splitn(2, "\r\n\r\n")
        .nth(1)
        .unwrap_or("");
    
    if method == "GET" {
        // Getting a repo
        handle_get();
    } else if method == "PUT" {
        // Pushing a file to a specific repo
        let path_without_filename = path
            .splitn(2, '/')
            .nth(0)
            .unwrap_or("");

        if path_without_filename == "/update" {
            let file_name = path
                .splitn(2, '/')
                .nth(1)
                .unwrap_or("");
        
            handle_update_file(body, file_name, &stream, params,);
        }
    } else if method == "POST" {
        // Making a new dir/repo
        if path_without_query == "/repo/new" {
            handle_create_dir(params, &stream);
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

fn handle_get() {
    
}

fn handle_update_file(file_contents: &str, file_name: &str, stream: &TcpStream, params: Vec<(&str, &str)>) {
   let mut message = "";

    if file_name == "" {
        message = "Couldnt get a file name (might be a server error)";
        send_back(message, stream, 404);
        return;
    }

    if params.is_empty() {
        message = "Couldnt get the repo/dir name to which to push to";
        send_back(message, stream, 404);
        return;
    }


    let (name_key, name_value) = params.get(0).unwrap();

    if *name_key != "where" {
        message = "Couldnt get the repo/dir name to which to push to";
        send_back(message, stream, 404);
        return;
    }

    if !(Path::new(&(REPOS_DIR.to_owned() + "/" + name_value)).exists()) {
       message = "Dir/Repo with that name doesnt exist, create it before pushing";  
       send_back(message, stream, 404);
       return;
    } 
    
    // if no check failed then we update/create the file 
    if !(Path::new(&(REPOS_DIR.to_owned() + "/" + name_value + "/" + file_name)).exists()) {
        // if file exists update
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&(REPOS_DIR.to_owned() + "/" + name_value + "/" + file_name));

        file.as_mut()
            .expect("Failed to write to file")
            .write_all(file_contents.as_bytes());

        file.as_mut()
            .expect("Failed to flush file")
            .flush();

        message = "Sucessfully updated existing file";
        send_back(message, stream, 200);
        return;
    } else {
        // if file doesnt exist create it

    }
}

fn handle_create_dir(params: Vec<(&str, &str)>, stream: &TcpStream) {
    let mut message = "";

    if params.is_empty() {
        message = "Couldnt get the name the new dir/repo"; 
        send_back(message, stream, 404);
        return;
    } 


    let (name_key, name_value) = params.get(0).unwrap();

    if *name_key == "name" {
        println!("Got a name");
        // check if the actual name is just empty
        if *name_value == "" {
            message = "No dir/repo name given";
            send_back(message, stream, 404);
            return;
        }

        // check if dir already exists
        if Path::new(&(REPOS_DIR.to_owned() + "/" + name_value)).exists() {
            message = "Dir/Repo with the same name already exists";
            send_back(message, stream, 404);
            return;
        }

        // after checking if everything is valid we cna create it
        match fs::create_dir(&(REPOS_DIR.to_owned() + "/" + name_value)) {
            Ok(()) => {
                message = "Succesfully created new dir/repo";
                send_back(message, stream, 201);
            }
            Err(e) => {
               println!("Error when creating new dir/repo: {e}") ;

               message = "Internal Server Error";
               send_back(message, stream, 500);
            }
        }
    } else {
        // send back 404 instantly
        message = "Couldnt get the name the new dir/repo"; 
        send_back(message, stream, 404);
        return;
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
