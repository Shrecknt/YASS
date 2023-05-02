use minecraft_utilities::{resolve_address, Client, Ping, ServerAddress};
use std::{error::Error, time::Duration};
use tokio::time::timeout;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut addr: ServerAddress = ServerAddress::try_from("mc.h4k.rs").unwrap();
    addr = resolve_address(&addr).await?.into();

    const SEND_HOSTNAME: &str = "shrecked.dev";
    const SEND_PORT: u16 = 25565;
    const SEND_USERNAME: &str = "Shrecknt";

    let send_protocol_version = minecraft_utilities::parse_version("1.19.4").unwrap();

    let ping_response_future = Ping::ping(
        &addr.host,
        Some(addr.port),
        Some(send_protocol_version.try_into().unwrap()),
        Some(SEND_HOSTNAME),
        Some(SEND_PORT),
    );
    let ping_response = timeout(Duration::from_millis(1000), ping_response_future).await??;
    let protocol_version: i32 = Ping::get_protocol_version(&ping_response).unwrap();

    let mut client: Client = Client::connect(&addr.host, Some(addr.port)).await?;
    let online_mode_future = client.check_online_mode(
        Some(protocol_version as usize),
        Some(SEND_HOSTNAME),
        Some(SEND_PORT),
        Some(SEND_USERNAME),
    );
    let online_mode = timeout(Duration::from_millis(1000), online_mode_future).await??;

    println!(
        "Got info | protocol_version: {}, online_mode: {:?}",
        protocol_version, online_mode
    );

    Ok(())
}
