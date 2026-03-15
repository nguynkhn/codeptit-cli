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
pub struct SubmissionResponse {
    #[serde(rename = "solution_id")]
    submission_id: u32,
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
                .and(line.strip_suffix(block_comment_end))
        })
        .map(|line| line.trim().to_owned())
        .or(fallback))
}

pub fn send(api: crate::codeptit::api::Api, submission: Submissison) -> anyhow::Result<u32> {
    let form = reqwest::blocking::multipart::Form::new()
        .text("course_id", submission.course_id.to_string())
        .text("question", submission.question_code)
        .text("compiler", (submission.language as u8).to_string());
    let form = match submission.code {
        SubmissionCode::File(path) => form.file("code_file", path)?,
        SubmissionCode::Source(source) => form.text("source_code", source),
    };

    let response: SubmissionResponse = api
        .request(reqwest::Method::POST, "/solutions")
        .multipart(form)
        .send()?
        .json()?;

    Ok(response.submission_id)
}
