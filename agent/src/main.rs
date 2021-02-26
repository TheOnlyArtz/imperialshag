mod socket;

const IP: &'static str = "0.0.0.0";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = socket::connect_to_cnc(IP, PORT).await?;

    Ok(())
}
