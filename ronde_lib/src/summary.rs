use crate::runner::CommandResult;
use serde_derive::Serialize;

/// Summary of the command results
#[derive(Debug, Serialize)]
pub struct Summary {
    /// Number of successful commands
    pub nb_ok: u32,
    /// Number of failed commands
    pub nb_err: u32,
}

impl Summary {
    /// Get Summary of the command results
    pub fn from_results(results: &Vec<CommandResult>) -> Summary {
        let mut nb_ok = 0;
        let mut nb_err = 0;
        for result in results {
            match &result.result {
                Ok(_) => nb_ok += 1,
                Err(_) => nb_err += 1,
            }
        }
        Summary { nb_ok, nb_err }
    }

    /// Is the summary ok?
    pub fn _is_ok(&self) -> bool {
        self.nb_err == 0
    }
}
