// author - Patryk Jędrzejczak

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod request_parsing;

use request_parsing::{try_parse_request};

// Type of the database of the key-value pairs.
pub type Db = Arc<Mutex<HashMap<String, String>>>;

// Struct keeping data of a single task. Its only purpose is
// simplifying definitions of some functions.
pub struct TaskData {
    socket: TcpStream,
    db: Db
}

impl TaskData {
    pub fn new(socket: TcpStream, db: Db) -> Self {
        TaskData { socket, db }
    }
}

impl StoreRequest {
    fn new(key: String, value: String) -> Self {
        StoreRequest { key, value }
    }
}

impl LoadRequest {
    fn new(key: String) -> Self {
        LoadRequest { key }
    }
}

pub enum Request {
    Store(StoreRequest),
    Load(LoadRequest)
}

pub struct StoreRequest {
    key: String,
    value: String
}

pub struct LoadRequest {
    key: String
}

// Error returned when something goes wrong during a task's work.
// We do not care what really happened because in every case we just
// finish the task and close the connection with the client.
#[derive(Debug)]
pub struct TaskError;

// Handles receiving requests from a single client.
// When execution of the function ends, connection also ends.
pub async fn handle_connection(mut data: TaskData) {
    static BUF_SIZE: usize = 1024;
    let mut buf = vec![0; BUF_SIZE];
    let mut message = String::new(); // Fragment of the message read so far.

    loop {
        match data.socket.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(read_num) => {
                buf[0..read_num].iter().for_each(|byte| message.push(*byte as char));

                if process_message(&mut message, &mut data).await.is_err() {
                    return
                }
            }
        }
    }
}

// Processes message until it has a prefix being a STORE or LOAD request.
// Returns TaskError, if message is for sure incorrect.
async fn process_message(message: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    loop {
        match try_parse_request(message) {
            Err(_) => return Err(TaskError),
            Ok(None) => return Ok(()),
            Ok(Some(request)) => process_request(request, data).await?
        }
    }
}

async fn process_request(request: Request, data: &mut TaskData) -> Result<(), TaskError> {
    match request {
        Request::Store(request) => process_store_request(request, data).await,
        Request::Load(request) => process_load_request(request, data).await
    }
}

async fn process_store_request(request: StoreRequest, data: &mut TaskData) -> Result<(), TaskError> {
    match data.db.lock() {
        Ok(mut db) => {
            db.insert(request.key, request.value);
        },
        Err(_) => return Err(TaskError)
    }

    send_done_response(&mut data.socket).await
}

async fn process_load_request(request: LoadRequest, data: &mut TaskData) -> Result<(), TaskError> {
    let value;
    match data.db.lock() {
        Ok(db) => {
            value = db.get(&request.key).map(|val| val.clone());
        },
        Err(_) => return Err(TaskError)
    }

    match value {
        None => send_not_found_response(&mut data.socket).await,
        Some(value) => send_found_response(&mut data.socket, value).await
    }
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
