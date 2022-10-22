// Patryk Jędrzejczak

use tokio::{net::{TcpListener, TcpStream}, io::AsyncReadExt};

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
    let mut buf = vec![0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return,
            Ok(bytes_read) => {
                let mut chars = String::new();
                for i in 0..bytes_read {
                    chars.push(buf[i].clone() as char);
                }
                println!("Read: {chars}");
            }
            Err(_) => return
        }
    }
}