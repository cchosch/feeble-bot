
use crate::PROD;
use anyhow::Result;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};

pub fn init_logger() -> Result<Handle> {
    let config = if PROD {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%m/%d %H:%M:%S%.3f %:z)} {f}:{L} {h([{l}]):7} {m}{n}",
            )))
            .build("./log.log")?;
        Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Warn))?
    } else {
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%H:%M:%S%.3f)} {f}:{L} {h([{l}]):7} {m}{n}",
            )))
            .build();
        Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Info))?
    };

    Ok(log4rs::init_config(config)?)
}