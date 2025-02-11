use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "keylog")]
#[command(version, about = "Control keylogger behaviour", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = String::from("/tmp/keylog.socket"))]
    socket: String,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[derive(Debug, Serialize, Deserialize)]
enum Commands {
    /// Record keystrokes
    Record {},
    /// Save keystrokes to file and stop recording
    Save {
        /// File to save the recording to
        #[arg(short, long)]
        file: String,
    },
    /// Replay recorded keystrokes
    Replay {
        /// File to replay keystrokes from
        #[arg(short, long)]
        file: String,
    },
    Pause {},
    Resume {},
}
