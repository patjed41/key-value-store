// author - Patryk Jędrzejczak

use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use key_value_store::{Db, TaskData};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:5555").await.unwrap();

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        tokio::spawn(async move {
            key_value_store::handle_connection(TaskData::new(socket, db)).await;
        });
    }
}
