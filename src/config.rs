#[serde_with::serde_as]
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub access_token: Option<String>,

    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub timeout: std::time::Duration,

    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub poll_interval: std::time::Duration,

    pub max_retries: u32,

    #[serde(rename = "course")]
    pub course_id: Option<crate::codeptit::api::ApiId>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            access_token: None,
            timeout: std::time::Duration::from_secs(5),
            poll_interval: std::time::Duration::from_secs(1),
            max_retries: 5,
            course_id: None,
        }
    }
}

impl Config {
    pub fn path() -> anyhow::Result<std::path::PathBuf> {
        let config_dir = dirs::config_dir().ok_or(anyhow::anyhow!("Not supported"))?;
        Ok(config_dir.join("codeptit-cli").join("config.toml"))
    }

    pub fn load(args: &crate::cli::Args) -> anyhow::Result<Self> {
        let config_path = Self::path()?;
        let config_file =
            <figment::providers::Toml as figment::providers::Format>::file(config_path);

        let config: Self = figment::Figment::new()
            .merge(figment::providers::Serialized::defaults(Self::default()))
            .merge(config_file)
            .merge(figment::providers::Serialized::defaults(args))
            .extract()?;
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn is_logged_in(&self) -> bool {
        self.access_token.is_some()
    }
}
