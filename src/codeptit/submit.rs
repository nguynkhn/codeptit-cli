#[derive(clap::ValueEnum, Clone)]
#[value(rename_all = "lowercase")]
pub enum Language {
    C = 1,
    Cpp = 2,
    Python = 3,
    CSharp = 4,
    Java = 5,
}

pub enum SubmissionCode {
    File(std::path::PathBuf),
    Source(String),
}

pub struct Submissison {
    course_id: u32,
    question_code: String,
    language: Language,
    code: SubmissionCode,
}

impl Submissison {
    pub fn new(
        course_id: u32,
        question_code: String,
        language: Language,
        code: SubmissionCode,
    ) -> Self {
        Self {
            course_id,
            question_code,
            language,
            code,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SubmissionSendResponse {
    #[serde(rename = "solution_id")]
    submission_id: u32,
}

#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, serde::Deserialize)]
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

#[derive(serde::Deserialize)]
pub struct SubmissionStatusResponse {
    data: SubmissionStatus,
}

pub fn language(path: &std::path::PathBuf) -> Option<Language> {
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    Some(match extension {
        "c" => Language::C,
        "cpp" => Language::Cpp,
        "python" => Language::Python,
        "cs" => Language::CSharp,
        "java" => Language::Java,
        _ => return None,
    })
}

pub fn question_code(language: &Language, code: &SubmissionCode) -> anyhow::Result<Option<String>> {
    let (line_comment, block_comment_start, block_comment_end) = match language {
        Language::Python => ("#", "\"\"\"", "\"\"\""),
        _ => ("//", "/*", "*/"),
    };

    let (line, fallback) = match code {
        SubmissionCode::File(path) => {
            let file = std::fs::File::open(path)?;
            let mut reader = std::io::BufReader::new(file);

            let mut line = String::new();
            std::io::BufRead::read_line(&mut reader, &mut line)?;

            let filename = path
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default()
                .to_owned();
            (line, Some(filename))
        }
        SubmissionCode::Source(source) => {
            (source.lines().next().unwrap_or_default().to_owned(), None)
        }
    };

    Ok(line
        .trim()
        .strip_prefix(line_comment)
        .or_else(|| {
            line.strip_prefix(block_comment_start)
                .and_then(|line| line.strip_suffix(block_comment_end))
        })
        .map(|line| line.trim().to_owned())
        .or(fallback))
}

pub fn send(api: &crate::codeptit::api::Api, submission: Submissison) -> anyhow::Result<u32> {
    let form = reqwest::blocking::multipart::Form::new()
        .text("course_id", submission.course_id.to_string())
        .text("question", submission.question_code)
        .text("compiler", (submission.language as u8).to_string());
    let form = match submission.code {
        SubmissionCode::File(path) => form.file("code_file", path)?,
        SubmissionCode::Source(source) => form.text("source_code", source),
    };

    let response: SubmissionSendResponse = api
        .request(reqwest::Method::POST, "/solutions")
        .multipart(form)
        .send()?
        .json()?;

    Ok(response.submission_id)
}

pub fn status(
    api: &crate::codeptit::api::Api,
    submission_id: u32,
) -> anyhow::Result<SubmissionStatus> {
    let response: SubmissionStatusResponse = api
        .request(reqwest::Method::GET, &format!("/solutions/{submission_id}"))
        .send()?
        .json()?;

    Ok(response.data)
}
