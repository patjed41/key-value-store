// author - Patryk Jędrzejczak

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod request_parsing;

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
    while request_parsing::is_store_request(message)?
            || request_parsing::is_load_request(message)? {
        if request_parsing::is_store_request(message)? {
            process_store_request(message, data).await?;
        } else {
            process_load_request(message, data).await?;
        }
    }
    

    if !request_parsing::could_become_store_request(message)?
            && !request_parsing::could_become_load_request(message)? {
        return Err(TaskError)
    }

    Ok(())
}

// Processes single prefix of message that is a STORE request and
// deletes it from the message.
async fn process_store_request(message: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    let (key, value, rest) = request_parsing::split_store_request(message);
    *message = rest;

    match data.db.lock() {
        Ok(mut db) => {
            db.insert(key, value);
        },
        Err(_) => return Err(TaskError)
    }

    send_done_response(&mut data.socket).await?;

    Ok(())
}

// Processes single prefix of message that is a LOAD request and
// deletes it from the message.
async fn process_load_request(message: &mut String, data: &mut TaskData) -> Result<(), TaskError> {
    let (key, rest) = request_parsing::split_load_request(message);
    *message = rest;

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
