mod connection;
mod future;

use tokio::{
    net::TcpListener,
    sync::{Notify, RwLock},
    time::sleep,
    fs::read,
};
use std::{
    sync::Arc,
    time::Duration,
    fs::File,
    io::Write,
};


const DATA_FILE: &str = "/tmp/test.txt";
const DEFAULT_DATA: &[u8] = b"ffffff";
/**
* In milliseconds
*/
const FILE_REFRESH_INTERVAL: u64 = 200;


#[tokio::main]
async fn main() {
    // Initialize signals and set listening addresses
    let bind_addresses: Vec<&str> = vec!["[2a01:4f8:c013:2b4d::1]:42069", "128.140.61.250:42069"];
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

    // Create and write to data file (as this program should run as a service)
    let mut file = File::create(DATA_FILE).expect("Could not create data file");
    file.write_all(DEFAULT_DATA).expect("Could not write default values to data file");

    // Wait for file changes and for data transfer completion
    loop {
        let mut lock = colors.write().await;
        let contents = read(DATA_FILE).await.expect("Could not read from data file");
        let mut changed: bool = false;
        for i in 0..3 {
            let Ok(hex_value) = hex_chars_to_u8(contents[i * 2], contents[i * 2 + 1]) else {
                panic!("Data file structure is invalid");
            };
            if hex_value != lock[i] {
                changed = true;
                lock[i] = hex_value;
            }
        }
        if changed {
            signal.notify_waiters();
        }
        sleep(Duration::from_millis(FILE_REFRESH_INTERVAL)).await;
    }
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
