use clap::Parser;

#[derive(Parser)]
#[command(name = "keylog")]
#[command(version, about = "Server that executes keylogger commands", long_about = None)]
pub struct ServerCli {
    #[arg(short, long, default_value_t = String::from("/tmp/keylog.socket"))]
    pub socket: String,

    /// Log file location
    #[arg(short, long, default_value_t = String::from("/var/log/keylog.log"))]
    pub log_file: String,

    /// Log level verbosity [possible values: error, warn, info, debug, trace]
    #[arg(short='d', long, default_value = "info", value_parser, ignore_case=true)]
    pub log_lvl: log::LevelFilter
}
