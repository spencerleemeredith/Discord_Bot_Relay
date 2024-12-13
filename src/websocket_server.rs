use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use std::sync::Arc;

pub async fn run_websocket_server(ws_sender: Arc<Mutex<Option<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>>>>) {
    let addr = "127.0.0.1:8080".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, ws_sender.clone()));
    }
}

async fn handle_connection(stream: TcpStream, ws_sender: Arc<Mutex<Option<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>>>>) {
    let addr = stream.peer_addr().expect("Connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    *ws_sender.lock().await = Some(write);

    let read_future = read.for_each(|message| async {
        if let Ok(msg) = message {
            if msg.is_close() {
                println!("WebSocket connection closed: {}", addr);
            }
        }
    });

    read_future.await;
    *ws_sender.lock().await = None;
    println!("WebSocket connection closed: {}", addr);
}
