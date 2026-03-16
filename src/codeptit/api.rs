pub type ApiId = u32;

pub struct Api<'a> {
    client: reqwest::blocking::Client,
    config: &'a crate::config::Config,
}

impl<'a> Api<'a> {
    pub fn new(config: &'a crate::config::Config) -> Self {
        Self {
            config,
            client: Default::default(),
        }
    }

    pub fn request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}{endpoint}", crate::codeptit::API_URL);
        let request = self
            .client
            .request(method, url)
            .timeout(self.config.timeout);

        if let Some(access_token) = &self.config.access_token {
            return request.bearer_auth(access_token);
        }

        request
    }

    pub fn request_poll<F, T>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        callback: F,
    ) -> anyhow::Result<T>
    where
        F: Fn(reqwest::blocking::RequestBuilder) -> anyhow::Result<T>,
    {
        let mut last_err = None;

        for _ in 0..=self.config.max_retries {
            let request = self.request(method.clone(), endpoint);

            match callback(request) {
                Ok(response) => return Ok(response),
                Err(err) => last_err = Some(err),
            }

            std::thread::sleep(self.config.poll_interval);
        }

        Err(last_err.unwrap_or(anyhow::anyhow!("Max retries exceeded")))
    }
}
