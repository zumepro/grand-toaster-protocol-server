mod connection;
mod future;

use tokio::{
    net::TcpListener,
    sync::{Notify, RwLock},
};
use std::{sync::Arc, io::Read};


const DATA_PIPE: &str = "/tmp/toaster_stream";
/**
* In milliseconds
*/
const DEBOUNCE_TIMEOUT: u64 = 200;


#[tokio::main]
async fn main() {
    // Initialize signals and set listening addresses
    let bind_addresses: Vec<&str> = vec!["[::1]:42069", "127.0.0.1:42069"];
    let signal = Arc::new(Notify::new());
    let colors: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(vec![255, 255, 255]));

    // Bind server sockets and spawn listening tasks
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

    // Create pipe
    let _ = tokio::fs::remove_file(DATA_PIPE).await;
    let output = tokio::process::Command::new("mkfifo")
        .arg("/tmp/toaster_stream")
        .output()
        .await.expect("Could not create the named pipe");
    if !output.status.success() {
        panic!("Could not create the named pipe");
    }

    // Wait for changes and for data transfer completion
    loop {
        let contents = read_from_pipe(DATA_PIPE).await.expect("Could not read from data pipe");
        let mut lock = colors.write().await;
        for i in 0..3 {
            let Ok(hex_value) = hex_chars_to_u8(contents[i * 2], contents[i * 2 + 1]) else {
                panic!("Data pipe stream structure is invalid");
            };
            lock[i] = hex_value;
        }
        signal.notify_waiters();
        tokio::time::sleep(tokio::time::Duration::from_millis(DEBOUNCE_TIMEOUT)).await;
    }
}


async fn read_from_pipe(path: &str) -> std::io::Result<Vec<u8>> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::File::open(path)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }).await?
}


fn hex_chars_to_u8(char1: u8, char2: u8) -> Result<u8, ()> {
    let digit1 = char_to_digit(char1 as char)?;
    let digit2 = char_to_digit(char2 as char)?;
    Ok((digit1 << 4) | digit2)
}


fn char_to_digit(c: char) -> Result<u8, ()> {
    match c {
        '0' ..= '9' => Ok(c as u8 - b'0'),
        'a' ..= 'f' => Ok(c as u8 - b'a' + 10),
        _ => Err(()),
    }
}
