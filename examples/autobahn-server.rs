use futures_tungstenite::{accept_async, tungstenite::Error};
use futures_util::{SinkExt, StreamExt};
use log::*;
use smol::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use tungstenite::Result;

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    info!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?;
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();

    smol::block_on(async {
        let addr = "127.0.0.1:9002";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        info!("Listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("connected streams should have a peer address");
            info!("Peer address: {}", peer);

            smol::spawn(accept_connection(peer, stream)).detach();
        }
    });
}
