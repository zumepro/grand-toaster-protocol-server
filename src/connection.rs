use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
    time::timeout,
    sync::{Notify, Mutex},
};
use std::{
    time::Duration,
    sync::Arc,
};


/**
* In seconds
*/
const SOCKET_IO_TIMEOUT: u64 = 60;


macro_rules! write_or_return {
    ($stream:ident, $to_write:expr) => {
        match timeout(Duration::from_secs(SOCKET_IO_TIMEOUT), $stream.write_all($to_write.as_bytes())).await {
            Ok(Ok(_)) => {},
            _ => return,
        }
    };
}


macro_rules! write_colors {
    ($stream:ident, $colors:ident) => {{
        let lock = $colors.lock().await;
        let current_colors: Vec<u8> = lock.clone();
        drop(lock);
        match timeout(Duration::from_secs(SOCKET_IO_TIMEOUT), $stream.write_all(&current_colors[..3])).await {
            Ok(Ok(_)) => {},
            _ => return,
        }
    }};
}


pub async fn handle_client(mut stream: TcpStream, signal: Arc<Notify>, colors: Arc<Mutex<Vec<u8>>>) {
    // Connection preface phase
    write_or_return!(stream, "GTP_4.2 SRV\r\n");
    let mut buf = String::new();
    let Ok((received, mut buf)) = read_line(&mut stream, &mut buf, 1).await else { return; };
    if received != "GTP_4.2 CLT" { return; }

    // Authentication phase
    let Ok((received, _)) = read_line(&mut stream, &mut buf, 1).await else { return; };
    if received != "i_am_a_toaster" { return; }
    write_or_return!(stream, "trust_me_bro\r\n");
    write_colors!(stream, colors);
}


/**
* Reads until \r\n line break.
* Will modify the contents of init: &mut String
* Returns (until_linebreak, after_linebreak)
*/
async fn read_line(stream: &mut TcpStream, init: &mut String, mut timeout_packets: u32) -> Result<(String, String), ()> {
    let mut buf = [0; 1024];
    timeout_packets += 1;
    for _ in 0..timeout_packets {
        if init.contains("\r\n") {
            let split: Vec<&str> = init.splitn(2, "\r\n").collect();
            return Ok((split[0].to_string(), split[1].to_string()));
        }
        let Ok(Ok(n)) = timeout(Duration::from_secs(SOCKET_IO_TIMEOUT), stream.read(&mut buf)).await else {
            return Err(());
        };
        init.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    Err(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn string_split() {
        let arr: Vec<&str> = "Hello,".split(",").take(2).collect();
        assert_eq!(arr, vec!["Hello", ""]);
        let arr: Vec<&str> = "Hello, there, world!".splitn(2, ",").collect();
        assert_eq!(arr, vec!["Hello", " there, world!"]);
    }
}
