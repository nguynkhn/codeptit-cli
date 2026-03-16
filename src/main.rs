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
        cli::Command::Login => {
            let username = match std::env::var(USERNAME_VAR) {
                Ok(var) => var,
                _ => cliclack::input("Enter your username: ").interact()?,
            };
            let password = match std::env::var(PASSWORD_VAR) {
                Ok(var) => var,
                _ => cliclack::password("Enter your password: ").interact()?,
            };

            let spinner = cliclack::spinner();
            spinner.start("Logging in...");

            let request = codeptit::login::LoginRequest { username, password };
            let response = api.login(request)?;

            config.access_token = Some(response.access_token);
            config.save()?;

            spinner.stop("Login successfully");
        }

        cli::Command::Course { course_id } => {
            if !config.is_logged_in() {
                cliclack::log::error("Please login first")?;
                std::process::exit(1);
            }

            let spinner = cliclack::spinner();
            spinner.start("Fetching courses...");

            let courses = api.courses()?.courses;
            if courses.is_empty() {
                spinner.error("No courses to select");
                std::process::exit(1);
            }

            spinner.stop(format!("{} courses found", courses.len()));

            let course_id = if let Some(course_id) = course_id {
                if !courses.iter().any(|course| course.id == course_id) {
                    cliclack::log::error(format!("Invalid course ID [{course_id}]"))?;
                    std::process::exit(1);
                }

                course_id
            } else {
                let items: Vec<_> = courses
                    .into_iter()
                    .map(|course| {
                        (
                            course.id,
                            format!(
                                "[{}] {} - {}",
                                course.id, course.subject.code, course.subject.name
                            ),
                            "",
                            // course.semester.name,
                        )
                    })
                    .collect();
                let select: cliclack::Select<codeptit::api::ApiId> =
                    cliclack::select("Select a course")
                        .items(&items)
                        .filter_mode();

                (match config.course_id {
                    Some(course_id) => select.initial_value(course_id),
                    None => select,
                })
                .interact()?
            };

            config.course_id = Some(course_id);
            config.save()?;

            cliclack::log::info(format!("Course ID [{course_id}] selected"))?;
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
                cliclack::log::error("No courses selected")?;
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
                // TODO: language select
                let source: String = cliclack::input("Enter your code").multiline().interact()?;
                let code = codeptit::submit::SubmissionCode::Source(source);

                (language, code)
            };

            if language.is_none() {
                cliclack::log::error("No language selected")?;
                std::process::exit(1);
            }

            let language = language.unwrap();
            let question_code = question_code.or(util::question_code(&language, &code)?);

            if question_code.is_none() {
                cliclack::log::error("No question_code found")?;
                std::process::exit(1);
            }

            let question_code = question_code.unwrap();

            let spinner = cliclack::spinner();
            spinner.start("Submitting code...");

            let request = codeptit::submit::SubmissionSendRequest {
                course_id,
                question_code,
                language,
                code,
            };
            let response = api.submission_send(request)?;
            let submission_id = response.submission_id;

            spinner.stop(format!("Submission sent with ID [{submission_id}]"));

            let spinner = cliclack::spinner();
            spinner.start("Waiting for result...");

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
            spinner.stop(result);
        }
    }
    Ok(())
}
