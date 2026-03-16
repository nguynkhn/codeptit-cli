#[derive(serde::Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
}

impl<'a> crate::codeptit::api::Api<'a> {
    pub fn login(&self, request: LoginRequest) -> anyhow::Result<LoginResponse> {
        let response: LoginResponse = self
            .request(reqwest::Method::POST, "/auth/login")
            .json(&request)
            .send()?
            .json()?;

        Ok(response)
    }
}
