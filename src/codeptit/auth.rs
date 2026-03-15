#[derive(serde::Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginResponse {
    access_token: String,
}

pub fn login(api: &crate::codeptit::api::Api) -> anyhow::Result<String> {
    let username = std::env::var("CODEPTIT_USERNAME").or_else(|_| -> anyhow::Result<String> {
        let mut username = String::new();

        print!("Enter your username: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        std::io::stdin().read_line(&mut username)?;
        Ok(username.trim().to_owned())
    })?;
    let password = std::env::var("CODEPTIT_PASSWORD").or_else(|_| {
        print!("Enter your password: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        rpassword::read_password()
    })?;

    let request = LoginRequest { username, password };
    let response: LoginResponse = api
        .request(reqwest::Method::POST, "/auth/login")
        .json(&request)
        .send()?
        .json()?;

    Ok(response.access_token)
}
