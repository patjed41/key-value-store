// author - Patryk Jędrzejczak

use tokio::net::TcpListener;
use tokio::sync::mpsc;

use key_value_store::{TaskData};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:5555").await.unwrap();

    let (request_sender, request_receiver) = mpsc::channel(32);

    tokio::spawn(async move {
        key_value_store::run_request_manager(request_receiver).await;
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let sender_copy = request_sender.clone();
        tokio::spawn(async move {
            key_value_store::handle_connection(TaskData::new(socket, sender_copy)).await;
        });
    }
}
