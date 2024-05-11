use ronde_lib::history::History;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).await?;
    let history: History = serde_yaml::from_slice(&buffer)?;

    let json = serde_json::to_string(&history)?;
    stdout.write_all(json.as_bytes()).await?;

    Ok(())
}
