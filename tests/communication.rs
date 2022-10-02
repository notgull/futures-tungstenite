use futures_tungstenite::{accept_async, client_async, WebSocketStream};
use futures_util::{sink::SinkExt, StreamExt};
use log::*;
use smol::{
    channel::{bounded, Sender},
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tungstenite::Message;

async fn run_connection<S>(connection: WebSocketStream<S>, msg_tx: Sender<Vec<Message>>)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    info!("Running connection");
    let mut connection = connection;
    let mut messages = vec![];
    while let Some(message) = connection.next().await {
        info!("Message received");
        let message = message.expect("Failed to get message");
        messages.push(message);
    }
    msg_tx.send(messages).await.expect("Failed to send results");
}

#[test]
fn communication() {
    smol::block_on(async {
        let _ = env_logger::try_init();

        let (con_tx, con_rx) = bounded(1);
        let (msg_tx, msg_rx) = bounded(1);

        let f = async move {
            let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
            info!("Server ready");
            con_tx.send(()).await.unwrap();
            info!("Waiting on next connection");
            let (connection, _) = listener.accept().await.expect("No connections to accept");
            let stream = accept_async(connection).await;
            let stream = stream.expect("Failed to handshake with connection");
            run_connection(stream, msg_tx).await;
        };

        smol::spawn(f).detach();

        info!("Waiting for server to be ready");

        con_rx.recv().await.expect("Server not ready");
        let tcp = TcpStream::connect("127.0.0.1:12345").await.expect("Failed to connect");
        let url = "ws://localhost:12345/";
        let (mut stream, _) = client_async(url, tcp).await.expect("Client failed to connect");

        for i in 1..10 {
            info!("Sending message");
            stream.send(Message::Text(format!("{}", i))).await.expect("Failed to send message");
        }

        stream.close(None).await.expect("Failed to close");

        info!("Waiting for response messages");
        let messages = msg_rx.recv().await.expect("Failed to receive messages");
        assert_eq!(messages.len(), 10);
    });
}

#[test]
fn split_communication() {
    smol::block_on(async {
        let _ = env_logger::try_init();

        let (con_tx, con_rx) = bounded(1);
        let (msg_tx, msg_rx) = bounded(1);

        let f = async move {
            let listener = TcpListener::bind("127.0.0.1:12346").await.unwrap();
            info!("Server ready");
            con_tx.send(()).await.unwrap();
            info!("Waiting on next connection");
            let (connection, _) = listener.accept().await.expect("No connections to accept");
            let stream = accept_async(connection).await;
            let stream = stream.expect("Failed to handshake with connection");
            run_connection(stream, msg_tx).await;
        };

        smol::spawn(f).detach();

        info!("Waiting for server to be ready");

        con_rx.recv().await.expect("Server not ready");
        let tcp = TcpStream::connect("127.0.0.1:12346").await.expect("Failed to connect");
        let url = url::Url::parse("ws://localhost:12345/").unwrap();
        let (stream, _) = client_async(url, tcp).await.expect("Client failed to connect");
        let (mut tx, _rx) = stream.split();

        for i in 1..10 {
            info!("Sending message");
            tx.send(Message::Text(format!("{}", i))).await.expect("Failed to send message");
        }

        tx.close().await.expect("Failed to close");

        info!("Waiting for response messages");
        let messages = msg_rx.recv().await.expect("Failed to receive messages");
        assert_eq!(messages.len(), 10);
    });
}
