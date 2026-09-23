use std::time::Duration;

use super::local::run_command;
use super::{ExecOutput, Host, HostError};

/// A Crow-managed lab container, reached with `podman|docker exec -i <name>`.
/// File operations use the exec-based defaults on [`Host`].
pub struct ContainerHost {
    engine: String,
    name: String,
}

impl ContainerHost {
    pub fn new(engine: &str, name: &str) -> Self {
        Self { engine: engine.to_string(), name: name.to_string() }
    }
}

impl Host for ContainerHost {
    fn label(&self) -> String {
        format!("{}:{}", self.engine, self.name)
    }

    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
        let mut full: Vec<&str> = vec![self.engine.as_str(), "exec", "-i", self.name.as_str()];
        full.extend_from_slice(argv);
        run_command(&full, stdin, timeout).map_err(|e| match e {
            // The engine itself failing to find the container is a reachability
            // problem, not the inner command failing.
            HostError::Failed { stderr, .. } if stderr.contains("no such container") || stderr.contains("No such container") => {
                HostError::Unreachable(format!("{} container {} is not running", self.engine, self.name))
            }
            other => other,
        })
    }
}
