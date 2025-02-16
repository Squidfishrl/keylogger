use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "keylog")]
#[command(version, about = "Control keylogger behaviour", long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t = String::from("/tmp/keylog.socket"))]
    pub socket: String,

    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum Commands {
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
    /// Stop recording keystrokes
    Pause {},
    /// Continue recording keystrokes
    Resume {},
}
