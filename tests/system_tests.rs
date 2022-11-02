// author - Patryk Jędrzejczak

// THESE TESTS REQUIRE RUNNING SERVER IN ANOTHER TERMINAL!

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BUF_LEN: usize = 1024;

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn correct_store_request_works() {
    let correct_store_requests = vec![
        "STORE$$$", "STORE$k$$", "STORE$key$$", "STORE$$v$", "STORE$$value$",
        "STORE$k$v$", "STORE$key$value$", "STORE$qwertyuiopasdfghjklzxcvbnm$value$",
        "STORE$key$qwertyuiopasdfghjklzxcvbnm$","STORE$$$a"
    ];

    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();

    let mut buf = vec![0; BUF_LEN];

    for request in correct_store_requests {
        socket.write(request.as_bytes()).await.unwrap();
        let read_num = socket.read(&mut buf).await.unwrap();
        assert_eq!("DONE$".as_bytes(), &buf[0..read_num]);
    }
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn correct_store_request_sent_partially_works() {
    let store_request_fragments = vec![
        "STO", "RE$k", "ey$v", "alu", "e$"
    ];

    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();
    socket.set_nodelay(true).unwrap();

    let mut buf = vec![0; BUF_LEN];

    for request in store_request_fragments {
        socket.write(request.as_bytes()).await.unwrap();
    }

    let read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("DONE$".as_bytes(), &buf[0..read_num]);
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn correct_load_request_works() {
    let correct_load_requests = vec![
        "LOAD$$", "LOAD$k$", "LOAD$key$", "LOAD$qwertyuiopasdfghjklzxcvbnm$",
        "LOAD$a$a"
    ];

    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();

    let mut buf = vec![0; BUF_LEN];

    for request in correct_load_requests {
        socket.write(request.as_bytes()).await.unwrap();
        let read_num = socket.read(&mut buf).await.unwrap();
        assert!("NOTFOUND$".as_bytes() == &buf[0..read_num]
             || "FOUND$".as_bytes() == &buf[0..6]);
    }
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn correct_load_request_sent_partially_works() {
    let load_request_fragments = vec![
        "LOA", "D$spl", "itk", "ey$"
    ];

    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();
    socket.set_nodelay(true).unwrap();

    let mut buf = vec![0; BUF_LEN];

    for request in load_request_fragments {
        socket.write(request.as_bytes()).await.unwrap();
    }

    let read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("NOTFOUND$".as_bytes(), &buf[0..read_num]);
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn store_request_overrides_value() {
    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();

    let mut buf = vec![0; BUF_LEN];
    let mut read_num;

    socket.write("STORE$override$a$".as_bytes()).await.unwrap();
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("DONE$".as_bytes(), &buf[0..read_num]);

    socket.write("LOAD$override$".as_bytes()).await.unwrap();
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("FOUND$a$".as_bytes(), &buf[0..read_num]);

    socket.write("STORE$override$b$".as_bytes()).await.unwrap();
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("DONE$".as_bytes(), &buf[0..read_num]);

    socket.write("LOAD$override$".as_bytes()).await.unwrap();
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("FOUND$b$".as_bytes(), &buf[0..read_num]);
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn messages_containing_many_requests_work() {
    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();

    let mut buf = vec![0; BUF_LEN];
    let mut read_num;

    socket.write("STORE$mra$mrb$STORE$mrc$mrd$STORE$mre$mrf$".as_bytes()).await.unwrap();
    for _ in 0..3 {
        read_num = socket.read(&mut buf).await.unwrap();
        assert_eq!("DONE$".as_bytes(), &buf[0..read_num]);
    }

    socket.write("LOAD$mra$LOAD$mrc$LOAD$mre$".as_bytes()).await.unwrap();
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("FOUND$mrb$".as_bytes(), &buf[0..read_num]);
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("FOUND$mrd$".as_bytes(), &buf[0..read_num]);
    read_num = socket.read(&mut buf).await.unwrap();
    assert_eq!("FOUND$mrf$".as_bytes(), &buf[0..read_num]);
}

#[ignore]
#[tokio::test]
#[ntest::timeout(1000)]
async fn sending_incorrect_message_with_3_dollars_closes_connection() {
    let mut socket = TcpStream::connect("127.0.0.1:5555").await.unwrap();

    let mut long_request = String::new();
    for i in 1..1000 {
        if i % 300 == 0 {
            long_request.push('$')
        }
        long_request.push('a');
    }

    loop {
        match socket.write(long_request.as_bytes()).await {
            Ok(_) => continue,
            Err(_) => break
        };
    }
}
