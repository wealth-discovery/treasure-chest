use anyhow::Result;
use config::{Config, Environment, File};
use serde::Deserialize;

pub fn get<'de, T: Deserialize<'de>>() -> Result<T> {
    let config = Config::builder()
        .add_source(
            Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_")
                .list_separator(" "),
        )
        .add_source(File::with_name("config.yaml").required(false))
        .build()?;
    let result = config.try_deserialize()?;
    Ok(result)
}
