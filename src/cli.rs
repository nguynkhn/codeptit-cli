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
}
