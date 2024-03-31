use futures::future::join_all;

use ronde_lib::config::Config;
use ronde_lib::error::RondeError;
use ronde_lib::history::History;
use ronde_lib::html;
use ronde_lib::notification::check_and_send_notifications;
use ronde_lib::runner;
use ronde_lib::summary::Summary;

use libc::setgid;
use libc::setuid;

/// Display usage
fn usage() {
    println!("ronde version {}", env!("CARGO_PKG_VERSION"));
    println!("Monitor your servers and services with alerting and a simple status page");
    println!();
    println!("USAGE:");
    println!("    ronde <YamlConfigFile>");
    println!();
    println!("FLAGS:");
    println!("    -h, --help       Prints help information");
    println!();
    println!("ARGS:");
    println!("    <YamlConfigFile>    YAML Config file describing the services to monitor");
}

#[tokio::main]
/// Main function
async fn main() -> Result<(), RondeError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        usage();
        return Err(RondeError::CliError());
    }
    if args[1] == "-h" || args[1] == "--help" {
        usage();
        return Ok(());
    }

    let yaml_file = &args[1];
    let config = Config::load(yaml_file).await?;

    let results = join_all(config.commands.into_iter().map(runner::execute_command)).await;

    /* Stop running as root */
    if let Some(gid) = config.gid {
        let result = unsafe { setgid(gid) };
        if result != 0 {
            panic!("Failed to setgid to {}", gid);
        }
    }
    if let Some(uid) = config.uid {
        let result = unsafe { setuid(uid) };
        if result != 0 {
            panic!("Failed to setuid to {}", uid);
        }
    }

    let mut history = History::load(&config.history_file).await?;

    history.purge_from_results(&results);
    let summary = Summary::from_results(&results);
    history.update(results);
    history.recreate_tags();
    history.rotate();

    html::generate_json_files(&config.output_dir, summary, &history, "Ronde".to_string()).await?;
    html::generate_auxiliary_files(&config.output_dir).await?;

    if let Some(ref nconfig) = config.notifications {
        check_and_send_notifications(nconfig, &history).await?;
    }

    history.save(&config.history_file).await?;
    Ok(())
}
