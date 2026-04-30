mod config;
mod models;
mod handlers;
mod response;
mod storage;

use std::{
    net::{TcpListener},
    path::{Path},
    fs,
};

use crate::config::REPOS_DIR;
use crate::handlers::handle_connection;

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
