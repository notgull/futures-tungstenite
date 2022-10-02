use futures_tungstenite::{accept_async, client_async};
use smol::{channel::bounded, Async};
use std::net::{TcpListener, TcpStream};

#[test]
fn handshakes() {
    smol::block_on(async {
        let (tx, rx) = bounded(1);

        let f = async move {
            let addr = ([127, 0, 0, 1], 12345);
            let listener = Async::<TcpListener>::bind(addr).unwrap();
            tx.send(()).await.unwrap();
            while let Ok((connection, _)) = listener.accept().await {
                let stream = accept_async(connection).await;
                stream.expect("Failed to handshake with connection");
            }
        };

        smol::spawn(f).detach();

        rx.recv().await.expect("Failed to wait for server to be ready");
        let addr = ([127, 0, 0, 1], 12345);
        let tcp = Async::<TcpStream>::connect(addr).await.expect("Failed to connect");
        let url = url::Url::parse("ws://localhost:12345/").unwrap();
        let _stream = client_async(url, tcp).await.expect("Client failed to connect");
    });
}
