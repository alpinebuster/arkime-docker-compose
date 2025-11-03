use std::fs::File;
use chrono::Local;


pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let now = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let log_file_name = format!("main_{}.log", timestamp);
    let log_file = File::create(&log_file_name)?;

    fern::Dispatch::new()
		.level(log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(log_file)
        .chain(std::io::stdout())
        .apply()?;
    
    Ok(())
}
