mod connection;
mod future;

use tokio::{
    net::TcpListener,
    sync::{Notify, Mutex},
};
use std::{
    sync::Arc,
    cell::RefCell,
};


#[tokio::main]
async fn main() {
    let bind_addresses: Vec<&str> = vec!["[::1]:42069", "127.0.0.1:42069"];
    let signal = Arc::new(Notify::new());
    let colors: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![255, 255, 255]));

    for addr in bind_addresses {
        let listener = TcpListener::bind(addr).await.unwrap_or_else(|_| panic!("Could not listen on {}", addr));
        println!("Listening on {}", addr);

        let rx = signal.clone();
        let col = colors.clone();
        tokio::spawn(async move {
            loop {
                let (socket, _) = match listener.accept().await {
                    Ok(value) => value,
                    Err(_) => continue,
                };

                let rx = rx.clone();
                let col = col.clone();
                tokio::spawn(async move {
                    connection::handle_client(socket, rx, col).await;
                });
            }
        });
    }

    let _ = future::Pending::default().await;
}
