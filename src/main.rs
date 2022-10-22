// Patryk Jędrzejczak

use tokio::{net::{TcpListener, TcpStream}, io::AsyncReadExt};
use regex::Regex;

const MAX_MESSAGE_LENGTH: usize = 10000;

#[tokio::main]
async fn main() {
    let port = 5555;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0; MAX_MESSAGE_LENGTH];
    let mut request = String::new();

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return,
            Ok(read_num) => {
                for i in 0..read_num {
                    request.push(buf[i] as char);
                }

                if request.len() > MAX_MESSAGE_LENGTH {
                    return
                }

                if let Err(_) = process_request(&mut request) {
                    return
                }
            }
            Err(_) => return
        }
    }
}

fn process_request(request: &mut String) -> Result<(), regex::Error> {
    let is_store = is_store_request(request)?;
    if is_store {
        let (key, value, rest) = split_store_request(request);
        *request = rest;
        return Ok(())
    }
    
    let is_load = is_load_request(request)?;
    if is_load {
        let (key, rest) = split_load_request(request);
        *request = rest
    }
    Ok(())
}

fn is_store_request(request: &str) -> Result<bool, regex::Error> {
    let store_regex = Regex::new(r"^STORE\$[a-z]*\$[a-z]*\$")?;
    return Ok(store_regex.is_match(request))
}

fn is_load_request(request: &str) -> Result<bool, regex::Error> {
    println!("{request}");
    let load_regex = Regex::new(r"^LOAD\$[a-z]*\$")?;
    return Ok(load_regex.is_match(request))
}

fn split_store_request(request: &str) -> (String, String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, dol)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let value = request[dollars[1] + 1..dollars[2]].to_string();
    let rest = request[dollars[2] + 1..].to_string();
    (key, value, rest)
}

fn split_load_request(request: &str) -> (String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, dol)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let rest = request[dollars[1] + 1..].to_string();
    (key, rest)
}
