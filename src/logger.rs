// crates.io
use time::{format_description, UtcOffset};
use tracing::{info, Level};
use tracing_subscriber::fmt::time::OffsetTime;

pub fn init() {
    let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
    let timer = OffsetTime::new(
        UtcOffset::current_local_offset().unwrap(),
        format_description::parse(format).unwrap(),
    );
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_timer(timer)
        .with_thread_names(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Set default subscriber failed");
    info!("Logger initialized");
}
