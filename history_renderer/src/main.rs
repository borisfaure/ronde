use ronde_lib::history::History;
use ronde_lib::html;
use ronde_lib::summary::Summary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const HISTORY_FILE: &str = "history.yaml";
    const OUTPUT_DIR: &str = "out/";

    let history = History::load(&HISTORY_FILE.to_string()).await?;
    let summary: Summary = history.get_summary_from_latest();
    html::generate_auxiliary_files(OUTPUT_DIR).await?;
    html::generate_json_files(OUTPUT_DIR, summary, &history, "Ronde".to_string()).await?;

    Ok(())
}
