#[derive(clap::ValueEnum, Clone, Eq, PartialEq, serde::Serialize)]
#[value(rename_all = "lowercase")]
pub enum SubmissionLanguage {
    C = 1,
    Cpp = 2,
    Python = 3,
    CSharp = 4,
    Java = 5,
}

#[derive(serde::Serialize)]
pub enum SubmissionCode {
    File(std::path::PathBuf),
    Source(String),
}

#[derive(serde::Serialize)]
pub struct SubmissionSendRequest {
    pub course_id: crate::codeptit::api::ApiId,
    pub question_code: String,
    pub language: SubmissionLanguage,
    pub code: SubmissionCode,
}

#[derive(serde::Deserialize)]
pub struct SubmissionSendResponse {
    #[serde(rename = "solution_id")]
    pub submission_id: crate::codeptit::api::ApiId,
}

#[derive(serde::Deserialize)]
pub enum SubmissionVerdict {
    #[serde(rename = "AC")]
    Accepted,

    #[serde(rename = "WA")]
    WrongAnswer,

    #[serde(rename = "TLE")]
    TimeLimitExceeded,

    #[serde(rename = "MLE")]
    MemoryLimitExceeded,

    #[serde(rename = "RE")]
    RuntimeError,

    #[serde(rename = "CE")]
    CompilationError,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
pub struct SubmissionStatus {
    #[serde(rename = "result")]
    pub verdict: SubmissionVerdict,

    #[serde(rename = "run_time")]
    #[serde_as(as = "serde_with::DurationMilliSecondsWithFrac<f64>")]
    pub runtime_ms: std::time::Duration,

    #[serde(rename = "memory")]
    pub memory_kb: u32,

    #[serde(rename = "correct_test")]
    pub passed_test: u32,
    pub total_test: u32,
}

#[derive(serde::Serialize)]
pub struct SubmissionStatusRequest {
    #[serde(rename = "solution_id")]
    pub submission_id: crate::codeptit::api::ApiId,
}

#[derive(serde::Deserialize)]
pub struct SubmissionStatusResponse {
    #[serde(rename = "data")]
    pub status: SubmissionStatus,
}

impl<'a> crate::codeptit::api::Api<'a> {
    pub fn submission_send(
        &self,
        request: SubmissionSendRequest,
    ) -> anyhow::Result<SubmissionSendResponse> {
        let form = reqwest::blocking::multipart::Form::new()
            .text("course_id", request.course_id.to_string())
            .text("question", request.question_code)
            .text("compiler", (request.language as u8).to_string());
        let form = match request.code {
            SubmissionCode::File(path) => form.file("code_file", path)?,
            SubmissionCode::Source(source) => form.text("source_code", source),
        };

        let response: SubmissionSendResponse = self
            .request(reqwest::Method::POST, "/solutions")
            .multipart(form)
            .send()?
            .json()?;

        Ok(response)
    }

    pub fn submission_status(
        &self,
        request: SubmissionStatusRequest,
    ) -> anyhow::Result<SubmissionStatusResponse> {
        self.request_poll(
            reqwest::Method::GET,
            &format!("/solutions/{}", request.submission_id),
            |request| Ok(request.send()?.json()?),
        )
    }
}
