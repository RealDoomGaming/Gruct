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
        .chars()
        .take_while(|&char| char != '/')
        .collect::<String>();

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap();
    let query = path
        .splitn(2, '?')
        .nth(1)
        .unwrap_or("");
    let params: Vec<(&str, &str)> = query
        .split('&')
        .filter_map(|pair| pair.split_once("="))
        .collect();
    
    if method == "GET" {
        // Getting a repo
        handle_get();
    } else if method == "PUT" {
        // Pushing a file to a specific repo
        handle_put();
    } else if method == "POST" {
        // Making a new dir/repo
        handle_post();
    }

    let status_line = "HTTP/1.1 200 OK";
    let content = "Hellos :D";
    let con_length = content.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {con_length}\r\n\r\n{content}");
    
    stream.write_all(response.as_bytes())?;

    Ok(())
}

fn handle_get() {
    
}

fn handle_put() {

}

fn handle_post() {

}
