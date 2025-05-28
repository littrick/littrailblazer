use log::LevelFilter;
use std::io::Write;

pub fn log_init(level: Option<LevelFilter>) {
    let mut builder = env_logger::builder();

    builder.format(|buf, record| {
        let level_color = match record.level() {
            log::Level::Error => 31, // 红色
            log::Level::Warn => 33,  // 黄色
            log::Level::Info => 32,  // 绿色
            log::Level::Debug => 36, // 青色
            log::Level::Trace => 90, // 灰色
        };
        writeln!(
            buf,
            "\x1b[{}m[{}] [{}] [{}]\x1b[0m - {}",
            level_color,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Some(level) = level {
        builder.filter_level(level);
    }

    builder.init();
}
