#[derive(Parser)]
#[command(name = "keylog")]
#[command(version, about = "Server that executes keylogger commands", long_about = None)]
struct ServerCli {
    #[arg(short, long, default_value_t = String::from("/tmp/keylog.socket"))]
    socket: String,

    /// Log file location
    #[arg(short, long, default_value_t = String::from("/var/log/keylog.log"))]
    log_file: String,

    /// Log level verbosity [possible values: error, warn, info, debug, trace]
    #[arg(short='d', long, default_value = "info", value_parser, ignore_case=true)]
    log_lvl: log::LevelFilter
}
