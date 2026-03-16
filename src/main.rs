const USERNAME_VAR: &'static str = "CODEPTIT_USERNAME";
const PASSWORD_VAR: &'static str = "CODEPTIT_PASSWORD";

mod cli;
mod codeptit;
mod config;
mod util;

fn main() -> anyhow::Result<()> {
    let cli = <cli::Cli as clap::Parser>::parse();
    let mut config = config::Config::load(&cli.args)?;
    let api = codeptit::api::Api::new(&config);

    match cli.command {
        cli::Command::Login { logout } => {
            if logout {
                config.access_token = None;
                config.course_id = None;
                config.save()?;

                println!("Logged out successfully");
                return Ok(());
            }

            let username = match std::env::var(USERNAME_VAR) {
                Ok(var) => var,
                _ => dialoguer::Input::new()
                    .with_prompt("Enter your username")
                    .interact_text()?,
            };
            let password = match std::env::var(PASSWORD_VAR) {
                Ok(var) => var,
                _ => dialoguer::Password::new()
                    .with_prompt("Enter your password")
                    .interact()?,
            };

            println!("Logging in...");

            let request = codeptit::login::LoginRequest { username, password };
            let response = api.login(request)?;

            config.access_token = Some(response.access_token);
            config.course_id = None;
            config.save()?;

            println!("Login successfully");
        }

        cli::Command::Course { course_id } => {
            if !config.is_logged_in() {
                eprintln!("Please login first");
                std::process::exit(1);
            }

            println!("Fetching courses...");

            let courses = api.courses()?.courses;
            if courses.is_empty() {
                eprintln!("No courses to select");
                std::process::exit(1);
            }

            println!("{} courses found", courses.len());

            let course_id = if let Some(course_id) = course_id {
                if !courses.iter().any(|course| course.id == course_id) {
                    eprintln!("Invalid course ID [{course_id}]");
                    std::process::exit(1);
                }

                course_id
            } else {
                let items: Vec<_> = courses
                    .iter()
                    .map(|course| {
                        format!(
                            "[{}] {} - {}",
                            course.id, course.subject.code, course.subject.name
                        )
                    })
                    .collect();

                let select = dialoguer::Select::new()
                    .with_prompt("Select a course")
                    .items(&items);

                let course_id = config
                    .course_id
                    .and_then(|id| courses.iter().position(|course| course.id == id));
                let selection = (match course_id {
                    Some(course_id) => select.default(course_id),
                    _ => select,
                })
                .interact()?;

                courses[selection].id
            };

            config.course_id = Some(course_id);
            config.save()?;

            println!("Course ID [{course_id}] selected");
        }

        cli::Command::Submit {
            course_id,
            question_code,
            language,
            file,
            stdin,
        } => {
            let course_id = course_id.or(config.course_id);
            if course_id.is_none() {
                eprintln!("No courses selected");
                std::process::exit(1);
            }

            let course_id = course_id.unwrap();
            let (language, code) = if stdin {
                let mut buffer = Vec::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();

                std::io::Read::read_to_end(&mut handle, &mut buffer)?;
                let source = String::from_utf8(buffer)?;
                let code = codeptit::submit::SubmissionCode::Source(source);

                (language, code)
            } else if let Some(path) = file {
                let language = util::language(&path);
                let code = codeptit::submit::SubmissionCode::File(path);

                (language, code)
            } else {
                eprintln!("Please send your code");
                std::process::exit(1);
            };

            if language.is_none() {
                eprintln!("No language selected");
                std::process::exit(1);
            }

            let language = language.unwrap();
            let question_code = question_code.or(util::question_code(&language, &code)?);

            if question_code.is_none() {
                eprintln!("No question_code found");
                std::process::exit(1);
            }

            let question_code = question_code.unwrap();

            println!("Submitting code...");

            let request = codeptit::submit::SubmissionSendRequest {
                course_id,
                question_code,
                language,
                code,
            };
            let response = api.submission_send(request)?;
            let submission_id = response.submission_id;

            println!("Submission sent with ID [{submission_id}]");
            println!("Waiting for result...");

            let request = codeptit::submit::SubmissionStatusRequest { submission_id };
            let response = api.submission_status(request)?;

            let result = match response.status.verdict {
                codeptit::submit::SubmissionVerdict::Accepted => "✓ Accepted!",
                codeptit::submit::SubmissionVerdict::WrongAnswer => "✗ Wrong Answer!",
                codeptit::submit::SubmissionVerdict::TimeLimitExceeded => "✗ Time Limit Exceeded!",
                codeptit::submit::SubmissionVerdict::MemoryLimitExceeded => {
                    "✗ Memory Limit Exceeded!"
                }
                codeptit::submit::SubmissionVerdict::RuntimeError => "✗ Runtime Error!",
                codeptit::submit::SubmissionVerdict::CompilationError => "✗ Compilation Error!",
            };
            println!("{result}");
        }
    }
    Ok(())
}
