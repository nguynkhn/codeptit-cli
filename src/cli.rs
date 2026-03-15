#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Login to CodePTIT
    Login,
    /// Select a course or display a list of courses
    Course {
        id: Option<u32>,
    },
    /// Submit your code
    Submit {
        #[arg(short, long = "question")]
        question_code: Option<String>,
        #[arg(conflicts_with = "stdin")]
        file: Option<std::path::PathBuf>,
        #[arg(short, long = "course")]
        course_id: Option<u32>,
        #[arg(short, long, value_enum)]
        language: Option<crate::codeptit::submit::Language>,
        #[arg(long, requires = "language", conflicts_with = "file")]
        stdin: bool,
    }
}
