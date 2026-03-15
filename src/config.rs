#[serde_with::serde_as]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub access_token: Option<String>,
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub timeout: std::time::Duration,
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub retry_interval: std::time::Duration,
    pub max_retries: u32,
    #[serde(rename = "course")]
    pub course_id: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            access_token: None,
            timeout: std::time::Duration::from_secs(5),
            retry_interval: std::time::Duration::from_secs(1),
            max_retries: 5,
            course_id: None,
        }
    }
}

impl Config {
    pub fn path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap()
            .join("codeptit-cli")
            .join("config.toml")
    }

    pub fn load() -> anyhow::Result<Self> {
        if let Ok(content) = std::fs::read_to_string(Self::path()) {
            let config = toml::from_str(&content)?;
            return Ok(config);
        }

        Ok(Default::default())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
