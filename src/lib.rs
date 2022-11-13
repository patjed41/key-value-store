// author - Patryk Jędrzejczak

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};
use tokio::fs::{File};

mod request_parsing;

type RequestSender = mpsc::Sender<RequestCommand>;
type RequestReceiver = mpsc::Receiver<RequestCommand>;
type RequestResponder<T> = oneshot::Sender<Result<T, TaskError>>;
use request_parsing::{try_parse_request};

// Struct keeping data of a single task. Its only purpose is
// simplifying definitions of some functions.
pub struct TaskData {
    socket: TcpStream,
    request_sender: RequestSender
}

impl TaskData {
    pub fn new(socket: TcpStream, request_sender: RequestSender) -> Self {
        TaskData { socket, request_sender }
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

// Command sent to request manager after reading a request from the client.
// Request manager answers using responder attribute.
pub enum RequestCommand {
    Store {
        request: StoreRequest,
        responder: RequestResponder<()>
    },
    Load {
        request: LoadRequest,
        responder: RequestResponder<Option<String>>
    }
}

impl RequestCommand {
    fn new_store(request: StoreRequest, responder: RequestResponder<()>) -> Self {
        RequestCommand::Store { request, responder }
    }

    fn new_load(request: LoadRequest, responder: RequestResponder<Option<String>>) -> Self {
        RequestCommand::Load { request, responder }
    }
}

// Error returned when something goes wrong during a task's work.
// We do not care what really happened because in every case we just
// finish the task and close the connection with the client.
#[derive(Debug)]
pub struct TaskError;

// Request manager
// It is the only task that has access to the file system. Other tasks ask
// request manager to execute a request (request manager receives them
// through request_receiver parameter). Request manager executes a request
// by creating/reading a file and sends answer to the asking task.
pub async fn run_request_manager(mut request_receiver: RequestReceiver) {
    static PATH_TO_DB: &str = "database";
    tokio::fs::create_dir_all(PATH_TO_DB).await.unwrap();


    while let Some(request_command) = request_receiver.recv().await {
        match request_command {
            RequestCommand::Store { request, responder } => {
                handle_store(format!("{}/key-{}", PATH_TO_DB, request.key), request.value, responder).await;
            },
            RequestCommand::Load { request, responder } => {
                handle_load(format!("{}/key-{}", PATH_TO_DB, request.key), responder).await;
            }
        }
    }
}

async fn handle_store(filename: String, value: String, responder: RequestResponder<()>)  {
    match File::create(filename).await {
        Err(_) => {
            responder.send(Err(TaskError)).unwrap_or(());
        },
        Ok(mut file) => {
            if file.write_all(value.as_bytes()).await.is_err() {
                responder.send(Err(TaskError)).unwrap_or(());
            } else {
                responder.send(Ok(())).unwrap_or(());
            }
        }
    }
}

async fn handle_load(filename: String, responder: RequestResponder<Option<String>>)  {
    match File::open(filename).await {
        Err(_) => {
            responder.send(Ok(None)).unwrap_or(());
        },
        Ok(mut file) => {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).await.is_err() {
                responder.send(Err(TaskError)).unwrap_or(());
            } else {
                let mut value = String::new();
                buf[0..buf.len()].iter().for_each(|byte| value.push(*byte as char));
                responder.send(Ok(Some(value))).unwrap_or(());
            }
        }
    }
}

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
    let (response_sender, response_receiver) = oneshot::channel();
    let request_command = RequestCommand::new_store(request, response_sender);
    if data.request_sender.send(request_command).await.is_err() {
        return Err(TaskError)
    }

    match response_receiver.await {
        Ok(Ok(())) => send_done_response(&mut data.socket).await,
        _ => Err(TaskError)
    }
}

// Processes single prefix of message that is a LOAD request and
// deletes it from the message.
async fn process_load_request(request: LoadRequest, data: &mut TaskData) -> Result<(), TaskError> {
    let (response_sender, response_receiver) = oneshot::channel();
    let request_command = RequestCommand::new_load(request, response_sender);
    if data.request_sender.send(request_command).await.is_err() {
        return Err(TaskError)
    }

    match response_receiver.await {
        Ok(Ok(None)) => send_not_found_response(&mut data.socket).await,
        Ok(Ok(Some(value))) => send_found_response(&mut data.socket, value).await,
        _ => Err(TaskError)
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
