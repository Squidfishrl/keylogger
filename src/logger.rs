use std::fs::File;
use std::io::Write;

pub fn init_logger(log_fname: &str, log_lvl: log::LevelFilter) -> Result<(), &'static str> {

    //let file = match File::create(log_fname) {
    let file = match File::options().append(true).create(true).open(log_fname) {
        Ok(f) => f,
        Err(_) => return Err("Cannot create or open the log file")
    };

    env_logger::builder()
        .filter_level(log_lvl)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}][{}] - {}",
                chrono::Local::now().format("%Y-%m%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    Ok(())
}
