mod cli;
mod codeptit;
mod config;

fn main() -> anyhow::Result<()> {
    let mut config = config::Config::load()?;
    let api = codeptit::api::Api::new(&config)?;
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Login => {
            let access_token = codeptit::auth::login(&api)?;
            config.access_token = Some(access_token);
            config.save()?;

            println!("Login successfully");
        }

        cli::Command::Course { id } => {
            let courses = codeptit::course::fetch(&api)?;
            match id {
                Some(id) => {
                    let course = courses
                        .into_iter()
                        .find(|course| course.id == id)
                        .expect(&format!("No course with ID {id} found"));

                    config.course_id = Some(course.id);
                    config.save()?;

                    println!(
                        "Course ID {}: {} - {} selected",
                        course.id, course.subject.code, course.subject.name
                    );
                }
                _ => {
                    todo!();
                }
            };
        }

        cli::Command::Submit {
            course_id,
            question_code,
            language,
            file,
            stdin,
        } => {
            let course_id = course_id.or(config.course_id).expect("No course selected");
            let (language, code) = if stdin {
                let mut buffer = Vec::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();

                std::io::Read::read_to_end(&mut handle, &mut buffer)?;

                let source = String::from_utf8(buffer)?;
                (
                    language.unwrap(),
                    codeptit::submit::SubmissionCode::Source(source),
                )
            } else {
                let file = file.expect("No file specified");
                let language = codeptit::submit::language(&file).expect("Not supported language");
                (language, codeptit::submit::SubmissionCode::File(file))
            };
            let question_code = crate::codeptit::submit::question_code(&language, &code)?
                .or(question_code)
                .expect("No question code specified");

            let submission =
                crate::codeptit::submit::Submissison::new(course_id, question_code, language, code);
            let submission_id = crate::codeptit::submit::send(&api, submission)?;

            println!("Submission sent with ID {submission_id}");

            let status =
                (0..=config.max_retries).find_map(|_| {
                    match crate::codeptit::submit::status(&api, submission_id) {
                        Ok(status) => Some(status),
                        _ => {
                            std::thread::sleep(config.retry_interval);
                            None
                        }
                    }
                });

            match status {
                Some(_) => todo!(),
                _ => eprintln!("Max retries exceeded"),
            }
        }
    };
    Ok(())
}
