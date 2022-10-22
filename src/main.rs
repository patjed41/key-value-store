// Patryk Jędrzejczak

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, String>>>;

const MAX_MESSAGE_LENGTH: usize = 10000;

#[tokio::main]
async fn main() {
    let port = 5555;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await.unwrap();

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        tokio::spawn(async move {
            handle_connection(TaskData { socket, db }).await;
        });
    }
}

async fn handle_connection(mut data: TaskData) {
    let mut buf = vec![0; MAX_MESSAGE_LENGTH];
    let mut request = String::new();

    loop {
        match data.socket.read(&mut buf).await {
            Ok(0) => return,
            Ok(read_num) => {
                for i in 0..read_num {
                    request.push(buf[i] as char);
                }

                if request.len() > MAX_MESSAGE_LENGTH {
                    return
                }

                if let Err(_) = process_request(&mut request, &mut data).await {
                    return
                }
            }
            Err(_) => return
        }
    }
}

async fn process_request(request: &mut String, data: &mut TaskData) -> Result<(), RequestError> {
    let is_store = is_store_request(request)?;
    if is_store {
        if process_store_request(request, data).await.is_err() {
            return Err(RequestError)
        }
    }
    
    let is_load = is_load_request(request)?;
    if is_load {
        if process_load_request(request, data).await.is_err() {
            return Err(RequestError)
        }
    }

    Ok(())
}

async fn process_store_request(request: &mut String, data: &mut TaskData) -> Result<(), RequestError> {
    let (key, value, rest) = split_store_request(request);
    *request = rest;

    match data.db.lock() {
        Ok(mut db) => {
            db.insert(key, value);
        },
        Err(_) => return Err(RequestError)
    }

    send_done_response(data).await?;

    Ok(())
}

async fn process_load_request(request: &mut String, data: &mut TaskData) -> Result<(), RequestError> {
    let (key, rest) = split_load_request(request);
    *request = rest;

    let mut value = String::from("not_found");
    match data.db.lock() {
        Ok(db) => {
            if let Some(val) = db.get(&key) {
                value = val.clone();
            }
        },
        Err(_) => return Err(RequestError)
    }

    send_found_response(data, value).await?;

    Ok(())
}

fn is_store_request(request: &str) -> Result<bool, RequestError> {
    match Regex::new(r"^STORE\$[a-z]*\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(RequestError)
    }
}

fn is_load_request(request: &str) -> Result<bool, RequestError> {
    match Regex::new(r"^LOAD\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(RequestError)
    }
}

fn split_store_request(request: &str) -> (String, String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let value = request[dollars[1] + 1..dollars[2]].to_string();
    let rest = request[dollars[2] + 1..].to_string();
    (key, value, rest)
}

fn split_load_request(request: &str) -> (String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let rest = request[dollars[1] + 1..].to_string();
    (key, rest)
}

async fn send_done_response(data: &mut TaskData) -> Result<(), RequestError> {
    if data.socket.write("DONE".as_bytes()).await.is_err() {
        return Err(RequestError)
    }

    Ok(())
}

async fn send_found_response(data: &mut TaskData, value: String) -> Result<(), RequestError> {
    if value == "not_found" {
        if data.socket.write("NOTFOUND".as_bytes()).await.is_err() {
            return Err(RequestError)
        }
    }
    else {
        if data.socket.write(format!("FOUND${value}$").as_bytes()).await.is_err() {
            return Err(RequestError)
        }
    }

    Ok(())
}

struct RequestError;

struct TaskData {
    socket: TcpStream,
    db: Db
}
