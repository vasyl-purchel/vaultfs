use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Vault {
    pub token: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub vault: Vault,
}

impl Settings {
  pub fn new(config: &str) -> Result<Self, ConfigError> {
      let mut s = Config::new();
      s.merge(File::with_name(config))?;
      return s.try_into();
  }
}
