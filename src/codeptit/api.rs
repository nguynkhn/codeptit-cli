pub struct Api {
    client: reqwest::blocking::Client,
}

impl Api {
    pub fn new(config: &crate::config::Config) -> anyhow::Result<Self> {
        let mut builder = reqwest::blocking::Client::builder()
            .timeout(config.timeout);

        if let Some(access_token) = config.access_token.as_deref() {
            let bearer = format!("Bearer {access_token}");

            let mut headers = reqwest::header::HeaderMap::new();
            let mut value = reqwest::header::HeaderValue::from_str(&bearer)?;
            value.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, value);

            builder = builder.default_headers(headers);
        }

        let client = builder.build()?;

        Ok(Self { client })
    }

    pub fn request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let url = format!("https://code.ptit.edu.vn/api{endpoint}");
        self.client.request(method, url)
    }
}
