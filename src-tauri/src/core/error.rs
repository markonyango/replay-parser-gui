use std::{error::Error as StdError, path::PathBuf, string::FromUtf8Error};

/// Errors of the replay watcher.
///
/// Setup errors (`LogfileNotFound` to `WatchEvents`) stop the watcher; everything else
/// concerns a single replay, which is skipped.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParserAppError {
    #[error("Could not find logfile")]
    LogfileNotFound,
    #[error("Could not find replay folder")]
    ReplayFolderNotFound,
    #[error("Could not resolve the documents folder: {0}")]
    DocumentDir(#[source] tauri::Error),
    #[error("Could not acquire main window handle")]
    MainWindowMissing,
    #[error("Replay path has no parent directory: {}", .0.display())]
    NoReplayFolder(PathBuf),
    #[error("Could not watch the replay folder: {0}")]
    Watch(#[from] notify::Error),
    #[error("File watcher failed: {}", join(.0))]
    WatchEvents(Vec<notify::Error>),

    #[error("Could not read logfile {}: {source}", .path.display())]
    LogfileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Logfile is not valid UTF-8: {0}")]
    LogfileEncoding(#[from] FromUtf8Error),
    #[error("Error while parsing logfile: {0}")]
    LogfileParse(String),
    #[error("Logfile contains no game")]
    NoGameInLogfile,
    #[error("Replay path is not valid UTF-8: {}", .0.display())]
    NonUtf8ReplayPath(PathBuf),
    #[error("Could not parse replay: {0}")]
    ReplayParse(#[source] Box<dyn StdError + Send + Sync>),
    #[error("Could not resolve replay details: {0}")]
    ReplayEnrich(String),
    #[error("Replay and logfile describe different matches: {0}")]
    MatchMismatch(String),
    #[error("Could not read replay {}: {source}", .path.display())]
    ReplayRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Could not copy replay file to {}: {source}", .target.display())]
    CopyReplayFile {
        target: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Could not send game to the main window: {0}")]
    NotifyWindow(#[source] tauri::Error),
}

/// Failures while reporting a replay to the remote server. These are recorded
/// in the game's `status` rather than propagated.
#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("Could not reach the server: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Unexpected server response (HTTP {0}): {1}")]
    BadResponse(reqwest::StatusCode, String),
}

pub type ParserAppResult<T, E = ParserAppError> = Result<T, E>;

fn join(errors: &[notify::Error]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
