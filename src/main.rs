// author - Patryk Jędrzejczak

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Type of the database of the key-value pairs.
type Db = Arc<Mutex<HashMap<String, String>>>;

// Maximum size of request that is allowed by server. If server receives a longer
// message without a single prefix that is a correct request, it closes the
// connection with the sender.
const MAX_MESSAGE_LENGTH: usize = 10000;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:5555").await.unwrap();

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
    let mut request = String::new(); // Request fragment read so far.

    loop {
        if let Ok(read_num) = data.socket.read(&mut buf).await {
            buf[0..read_num].iter().for_each(|byte| request.push(*byte as char));

            if process_request(&mut request, &mut data).await.is_err() {
                return
            }
        } else { // The case where client closed the connection or socket.read failed.
            return
        }
    }
}

async fn process_request(request: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    if is_store_request(request)? {
        process_store_request(request, data).await?;
    }
    
    if is_load_request(request)? {
        process_load_request(request, data).await?;
    }

    if request.len() > MAX_MESSAGE_LENGTH {
        return Err(TaskError)
    }

    Ok(())
}

async fn process_store_request(request: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    let (key, value, rest) = split_store_request(request);
    *request = rest;

    match data.db.lock() {
        Ok(mut db) => {
            db.insert(key, value);
        },
        Err(_) => return Err(TaskError)
    }

    send_done_response(&mut data.socket).await?;

    Ok(())
}

async fn process_load_request(request: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    let (key, rest) = split_load_request(request);
    *request = rest;

    let mut value = String::from("not_found");
    match data.db.lock() {
        Ok(db) => {
            if let Some(val) = db.get(&key) {
                value = val.clone();
            }
        },
        Err(_) => return Err(TaskError)
    }

    if value == "not_found" {
        send_not_found_response(&mut data.socket).await?;
    } else {
        send_found_response(&mut data.socket, value).await?;
    }

    Ok(())
}

// Returns true if there exists a prefix of a request parameter
// that is a correct STORE request.
fn is_store_request(request: &str) -> Result<bool, TaskError> {
    match Regex::new(r"^STORE\$[a-z]*\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(TaskError)
    }
}

// Returns true if there exists a prefix of a request parameter
// that is a correct LOAD request.
fn is_load_request(request: &str) -> Result<bool, TaskError> {
    match Regex::new(r"^LOAD\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(TaskError)
    }
}

// Splits a string with a prefix that is a correct STORE request
// from STORE$key$value$rest to (key, value, rest).
fn split_store_request(request: &str) -> (String, String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let value = request[dollars[1] + 1..dollars[2]].to_string();
    let rest = request[dollars[2] + 1..].to_string();
    (key, value, rest)
}

// Splits a string with a prefix that is a correct LOAD request
// from LOAD$key$rest to (key, rest).
fn split_load_request(request: &str) -> (String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let rest = request[dollars[1] + 1..].to_string();
    (key, rest)
}

async fn send_done_response(socket: &mut TcpStream) -> Result<(), TaskError> {
    match socket.write("DONE$".as_bytes()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(TaskError)
    }
}

async fn send_found_response(socket: &mut TcpStream, value: String) -> Result<(), TaskError> {
    match socket.write(format!("FOUND${value}$").as_bytes()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(TaskError)
    }
}

async fn send_not_found_response(socket: &mut TcpStream) -> Result<(), TaskError> {
    match socket.write("NOTFOUND$".as_bytes()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(TaskError)
    }
}

// Error returned when something goes wrong during a task's work.
// We do not care what really happened because in every case we just
// finish the task and close the connection with the client.
struct TaskError;

// Struct keeping data of a single task. Its only purpose is
// simplifying definitions of some functions.
struct TaskData {
    socket: TcpStream,
    db: Db
}
