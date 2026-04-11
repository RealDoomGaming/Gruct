use std::{
    // BufReader and prelude -> traits and types which let us read and write to stream
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    error::Error,
};

mod constants {
    const REPOS_DIR: &str = "/var/lib/gruct-repos";
    const _LOGS_DIR: &str = "/var/log/gruct-logs";
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

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
    
    if method == "GET" {
        // Getting a repo
    } else if method == "PUT" {
        // Pushing a file to a specific repo
    } else if method == "POST" {
        // Making a new dir/repo
    }

    let status_line = "HTTP/1.1 200 OK";
    let content = "Hellos :D";
    let con_length = content.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {con_length}\r\n\r\n{content}");
    
    stream.write_all(response.as_bytes())?;

    Ok(())
}
