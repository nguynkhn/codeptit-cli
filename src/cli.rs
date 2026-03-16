const LOGO: &'static str = include_str!("../logo.ans");

#[derive(clap::Args, serde::Serialize)]
pub struct Args {
    #[arg(long, global = true, value_parser = humantime::parse_duration)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<std::time::Duration>,

    #[arg(long, global = true, value_parser = humantime::parse_duration)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval: Option<std::time::Duration>,

    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Login to CodePTIT
    Login,
    /// Select a course
    Course {
        course_id: Option<crate::codeptit::api::ApiId>,
    },
    // Problem,
    /// Submit your code
    Submit {
        #[arg(short, long = "course")]
        course_id: Option<u32>,

        #[arg(short, long = "question")]
        question_code: Option<String>,

        #[arg(short, long, value_enum)]
        language: Option<crate::codeptit::submit::SubmissionLanguage>,

        #[arg(conflicts_with = "stdin")]
        file: Option<std::path::PathBuf>,

        #[arg(long, requires = "language", conflicts_with = "file")]
        stdin: bool,
    },
}

#[derive(clap::Parser)]
#[command(about = LOGO.trim())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub args: Args,
}
