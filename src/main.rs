use minecraft_utilities::{resolve_address, Client, OnlineModeResults, Ping, ServerAddress};
use serde_json::Value;
use std::{error::Error, time::Duration};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::timeout;

static SEND_HOSTNAME: &str = "shrecked.dev";
static SEND_PORT: u16 = 25565;
static SEND_USERNAME: &str = "Shrecknt";

#[allow(dead_code)]
#[derive(Debug)]
pub struct PingResults {
    address: ServerAddress,
    protocol_version: i32,
    ping_response: Value,
    online_mode: OnlineModeResults,
    other: Option<String>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let queue: Vec<ServerAddress> = read_ips().await?; //pull_ips().await?;
    let mut index = 0usize;

    while index < queue.len() {
        println!("Checking: {}", queue[index]);

        let ping_future = check_server(&queue[index]);
        let res = timeout(Duration::from_millis(10000), ping_future).await;
        match res {
            Ok(res) => match res {
                Ok(res) => {
                    println!("res: {res:?}");
                }
                Err(err) => {
                    println!("Error: {err}");
                }
            },
            Err(err) => {
                println!("Error: {err}");
            }
        }

        index += 1;
    }

    Ok(())
}

async fn check_server(address: &ServerAddress) -> Result<PingResults, Box<dyn Error>> {
    let addr = ServerAddress::try_from(resolve_address(address).await?)?;

    let send_protocol_version = minecraft_utilities::parse_version("1.19.4")?;

    let ping_response_future = Ping::ping(
        &addr.host,
        Some(addr.port),
        Some(send_protocol_version.try_into()?),
        Some(SEND_HOSTNAME),
        Some(SEND_PORT),
    );
    let ping_response = timeout(Duration::from_millis(2000), ping_response_future).await??;
    let protocol_version: i32 = Ping::get_protocol_version(&ping_response)?;

    let mut client: Client = Client::connect(&addr.host, Some(addr.port)).await?;
    let online_mode_future = client.check_online_mode(
        Some(protocol_version),
        Some(SEND_HOSTNAME),
        Some(SEND_PORT),
        Some(SEND_USERNAME),
    );
    let (online_mode, other) = timeout(Duration::from_millis(2000), online_mode_future).await??;

    Ok(PingResults {
        address: addr,
        protocol_version,
        ping_response,
        online_mode,
        other,
    })
}

#[allow(dead_code)]
async fn pull_ips() -> Result<Vec<ServerAddress>, Box<dyn Error>> {
    let ips_url = "https://github.com/mat-1/minecraft-scans/blob/main/ips?raw=true";

    println!("Requesting binary from github...");
    let buf = reqwest::get(ips_url).await?.bytes().await?;

    let mut res: Vec<ServerAddress> = vec![];

    println!("Parsing binary data...");
    let mut count = 0;
    for chunk in buf.chunks(6) {
        let port = u16::from(chunk[4]) * 256 + u16::from(chunk[5]);
        let append = format!(
            "{}.{}.{}.{}:{}",
            chunk[0], chunk[1], chunk[2], chunk[3], port
        );
        res.push(ServerAddress::try_from(append.as_str())?);
        count += 1;
    }
    println!("Found {} entries...", count);

    Ok(res)
}

async fn read_ips() -> Result<Vec<ServerAddress>, Box<dyn Error>> {
    let mut res = vec![];
    let file = File::open("ips.txt").await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        res.push(ServerAddress::try_from(line.as_str())?);
    }
    Ok(res)
}
