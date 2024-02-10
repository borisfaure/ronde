use rand::seq::SliceRandom;
use ronde_lib::{
    history::{CommandHistory, CommandHistoryEntry, History, HistoryError, TimeTag},
    runner::CommandOutput,
};
use std::vec::Vec;

const NB_COMMANDS: usize = 10;
const HISTORY_FILE: &str = "history.yaml";
const PERCENTAGE_OF_SUCCESS: f32 = 0.95;
const PERCENTAGE_OF_TIMEOUT: f32 = 0.2;
const PERCENTAGE_OF_COMMAND_ERROR: f32 = 0.05;

const LOREM_IPSUM: [&str; 103] = [
    "a",
    "accumsan",
    "adipiscing",
    "aenean",
    "aliquam",
    "amet",
    "ante",
    "aptent",
    "arcu",
    "at",
    "auctor",
    "augue",
    "bibendum",
    "condimentum",
    "congue",
    "consequat",
    "convallis",
    "curabitur",
    "curae",
    "dapibus",
    "diam",
    "dictumst",
    "dolor",
    "donec",
    "dui",
    "duis",
    "egestas",
    "eget",
    "eleifend",
    "elementum",
    "erat",
    "est",
    "eu",
    "facilisis",
    "feugiat",
    "fusce",
    "habitant",
    "himenaeos",
    "id",
    "in",
    "integer",
    "ipsum",
    "lacus",
    "laoreet",
    "lectus",
    "leo",
    "libero",
    "litora",
    "lorem",
    "magna",
    "malesuada",
    "massa",
    "mauris",
    "molestie",
    "nam",
    "nec",
    "netus",
    "nibh",
    "nisi",
    "non",
    "nostra",
    "nulla",
    "nullam",
    "nunc",
    "ornare",
    "pharetra",
    "phasellus",
    "placerat",
    "platea",
    "praesent",
    "pretium",
    "primis",
    "proin",
    "pulvinar",
    "purus",
    "quisque",
    "rutrum",
    "sagittis",
    "sed",
    "sem",
    "semper",
    "sit",
    "sociosqu",
    "sollicitudin",
    "suscipit",
    "suspendisse",
    "taciti",
    "tellus",
    "tempor",
    "tempus",
    "tincidunt",
    "tortor",
    "tristique",
    "ultrices",
    "ut",
    "vehicula",
    "vel",
    "velit",
    "vestibulum",
    "vitae",
    "vivamus",
    "volutpat",
    "vulputate",
];

fn generate_random_sentence() -> String {
    LOREM_IPSUM
        .choose_multiple(&mut rand::thread_rng(), 10)
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn generate_random_paragraph() -> String {
    let mut sentences: Vec<String> = Vec::new();
    for _ in 0..5 {
        sentences.push(generate_random_sentence());
    }
    sentences.join(". ")
}

const TIMESTAMPS: [&str; 50] = [
    "Tue, 30 Jan 2024 00:40:00 GMT",
    "Tue, 30 Jan 2024 01:41:22 GMT",
    "Wed, 31 Jan 2024 01:22:22 GMT",
    "Thu, 01 Feb 2024 01:33:33 GMT",
    "Fri, 02 Feb 2024 01:44:44 GMT",
    "Sat, 03 Feb 2024 01:55:55 GMT",
    "Sun, 04 Feb 2024 01:06:06 GMT",
    "Mon, 05 Feb 2024 01:00:00 GMT",
    "Tue, 06 Feb 2024 01:41:22 GMT",
    "Tue, 06 Feb 2024 18:49:41 GMT",
    "Tue, 06 Feb 2024 18:49:42 GMT",
    "Tue, 06 Feb 2024 18:49:43 GMT",
    "Tue, 06 Feb 2024 18:49:44 GMT",
    "Tue, 06 Feb 2024 19:49:44 GMT",
    "Tue, 06 Feb 2024 20:41:22 GMT",
    "Tue, 06 Feb 2024 21:11:31 GMT",
    "Tue, 06 Feb 2024 21:41:40 GMT",
    "Tue, 06 Feb 2024 22:41:59 GMT",
    "Tue, 06 Feb 2024 23:41:08 GMT",
    "Wed, 07 Feb 2024 00:00:00 GMT",
    "Wed, 07 Feb 2024 01:41:22 GMT",
    "Wed, 07 Feb 2024 07:19:22 GMT",
    "Wed, 07 Feb 2024 10:04:22 GMT",
    "Wed, 07 Feb 2024 17:14:22 GMT",
    "Wed, 07 Feb 2024 17:19:22 GMT",
    "Wed, 07 Feb 2024 18:04:22 GMT",
    "Wed, 07 Feb 2024 18:09:22 GMT",
    "Wed, 07 Feb 2024 18:34:22 GMT",
    "Wed, 07 Feb 2024 18:39:22 GMT",
    "Wed, 07 Feb 2024 18:44:21 GMT",
    "Wed, 07 Feb 2024 18:49:42 GMT",
    "Wed, 07 Feb 2024 18:49:43 GMT",
    "Wed, 07 Feb 2024 18:49:44 GMT",
    "Wed, 07 Feb 2024 18:54:22 GMT",
    "Wed, 07 Feb 2024 18:55:11 GMT",
    "Wed, 07 Feb 2024 18:56:33 GMT",
    "Wed, 07 Feb 2024 18:57:55 GMT",
    "Wed, 07 Feb 2024 19:04:04 GMT",
    "Wed, 07 Feb 2024 19:09:22 GMT",
    "Wed, 07 Feb 2024 19:14:22 GMT",
    "Wed, 07 Feb 2024 19:18:22 GMT",
    "Wed, 07 Feb 2024 19:19:22 GMT",
    "Wed, 07 Feb 2024 19:24:22 GMT",
    "Wed, 07 Feb 2024 19:29:22 GMT",
    "Wed, 07 Feb 2024 19:32:55 GMT",
    "Wed, 07 Feb 2024 19:34:22 GMT",
    "Wed, 07 Feb 2024 19:39:22 GMT",
    "Wed, 07 Feb 2024 19:44:21 GMT",
    "Wed, 07 Feb 2024 19:48:21 GMT",
    "Wed, 07 Feb 2024 19:49:43 GMT",
];

fn gen_command_history_entry(timestamp: &str) -> CommandHistoryEntry {
    let result = if rand::random::<f32>() < PERCENTAGE_OF_SUCCESS {
        Result::Ok(CommandOutput {
            exit: 0,
            stdout: generate_random_paragraph(),
            stderr: String::new(),
        })
    } else if rand::random::<f32>() < PERCENTAGE_OF_TIMEOUT {
        Result::Err(HistoryError::Timeout)
    } else if rand::random::<f32>() < PERCENTAGE_OF_COMMAND_ERROR {
        Result::Err(HistoryError::CommandError {
            exit: (1 + rand::random::<u8>()) as i32,
            stdout: generate_random_paragraph(),
            stderr: generate_random_paragraph(),
        })
    } else {
        Result::Err(HistoryError::Other {
            message: generate_random_sentence(),
        })
    };
    CommandHistoryEntry {
        timestamp: chrono::DateTime::parse_from_rfc2822(timestamp)
            .unwrap()
            .to_utc(),
        tag: TimeTag::Minute(0),
        result,
    }
}

fn gen_command_history(name: String) -> CommandHistory {
    let mut entries: Vec<CommandHistoryEntry> = Vec::with_capacity(TIMESTAMPS.len());
    for timestamp in TIMESTAMPS {
        entries.push(gen_command_history_entry(timestamp));
    }
    CommandHistory { name, entries }
}

fn gen_history(nb_commands: usize) -> History {
    let mut history = History::default();
    for n in 0..nb_commands {
        let command_history = gen_command_history(format!("command_{}", n));
        history.commands.push(command_history);
    }
    history.recreate_tags();
    history.rotate();
    history
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let history = gen_history(NB_COMMANDS);

    history.save(&HISTORY_FILE.to_string()).await?;

    Ok(())
}
