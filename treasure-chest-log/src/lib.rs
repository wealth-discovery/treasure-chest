use serde::Deserialize;
use std::{path::PathBuf, sync::Once};
use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{Layer, Registry, filter::Targets, fmt, layer::SubscriberExt};

pub type Level = LevelFilter;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub console: bool,
    pub file: bool,
    pub file_dir: PathBuf,
    pub targets: Vec<String>,
    pub level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            console: true,
            file: false,
            file_dir: PathBuf::from("logs"),
            targets: vec![],
            level: "info".to_string(),
        }
    }
}

static INIT: Once = Once::new();

pub fn init(config: &Config) {
    INIT.call_once(|| {
        let mut layers: Option<Box<dyn Layer<Registry> + Sync + Send>> = None;

        if !config.targets.is_empty() {
            let layer = Targets::new()
                .with_targets(config.targets.iter().map(|target| (target, Level::TRACE)));
            layers = Some(match layers {
                Some(l) => l.and_then(layer).boxed(),
                None => layer.boxed(),
            });
        }

        if config.console {
            let (writer, guard) = non_blocking(std::io::stdout());
            std::mem::forget(guard);
            let layer = fmt::layer().with_writer(writer);
            layers = Some(match layers {
                Some(l) => l.and_then(layer).boxed(),
                None => layer.boxed(),
            });
        }

        if config.file {
            let rolling_file = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_suffix("log")
                .build(&config.file_dir)
                .expect("initializing rolling file appender failed");
            let (writer, guard) = non_blocking(rolling_file);
            std::mem::forget(guard);
            let layer = fmt::layer().with_writer(writer);
            layers = Some(match layers {
                Some(l) => l.and_then(layer).boxed(),
                None => layer.boxed(),
            });
        }

        set_global_default(Registry::default().with(layers.expect("layers is not set"))).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        let config = Config {
            file: true,
            ..Default::default()
        };
        init(&config);

        tracing::trace!("trace");
        tracing::debug!("debug");
        tracing::info!("info");
        tracing::warn!("warn");
        tracing::error!("error");
    }
}
