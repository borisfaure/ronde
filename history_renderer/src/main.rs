use ronde_lib::history::History;
use ronde_lib::html;
use ronde_lib::summary::Summary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const HISTORY_FILE: &str = "history.yaml";
    const INDEX_FILE: &str = "out/index.html";
    const OUTPUT_DIR: &str = "out/";

    let history = History::load(&HISTORY_FILE.to_string()).await?;
    let summary: Summary = history.get_summary_from_latest();
    html::generate_auxiliary_files(OUTPUT_DIR).await?;
    let html = html::generate(summary, &history);
    tokio::fs::write(INDEX_FILE, html).await?;

    Ok(())
}
