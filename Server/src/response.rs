use crate::models::FileNode;
use std::{
    net::TcpStream,
    io::Write,
};
use serde_json;

pub fn send_back_keys(mut stream: &TcpStream, status_code: i32, keys: &str) {
    let message = keys;
    let message_len = message.len();

    let status_text = match status_code {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let resp = format!("HTTP/1.1 {status_code} {status_text}\r\nContent-Length: {message_len}\r\nContent-Type: text/plain\r\n\r\n{message}");

    stream.write_all(resp.as_bytes()).expect("Failed to Write to client");  
}


pub fn send_back_key(mut stream: &TcpStream, status_code: i32, key: &str) {
    let message = serde_json::to_string(&key).unwrap();
    let message_len = message.len();

    let status_text = match status_code {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let resp = format!("HTTP/1.1 {status_code} {status_text}\r\nContent-Length: {message_len}\r\nContent-Type: application/json\r\n\r\n{message}");

    stream.write_all(resp.as_bytes()).expect("Failed to Write to client");  
}

pub fn send_back_repo(mut stream: &TcpStream, status_code: i32, repo_path: FileNode) {
    let message = serde_json::to_string(&repo_path).unwrap();
    let message_len = message.len();

    let status_text = match status_code {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let resp = format!("HTTP/1.1 {status_code} {status_text}\r\nContent-Length: {message_len}\r\nContent-Type: application/json\r\n\r\n{message}");

    stream.write_all(resp.as_bytes()).expect("Failed to Write to client");  
}


pub fn send_back(message: &str, mut stream: &TcpStream, status_code: i32) {
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
