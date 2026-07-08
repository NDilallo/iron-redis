use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const BIND_ADDR: &str = "127.0.0.1:6379";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(BIND_ADDR).await?;
    println!("iron-redis listening on {BIND_ADDR}");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("accepted connection from {addr}");
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        eprintln!("connection {addr} closed with error: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("failed to accept connection: {e}");
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        stream.write_all(&buf[..n]).await?;
    }
}
