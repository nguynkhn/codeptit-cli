mod cli;
mod codeptit;
mod config;

fn main() -> anyhow::Result<()> {
    let mut config = config::Config::load()?;
    let api = codeptit::api::Api::new(&config)?;
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Login => {
            let access_token = codeptit::auth::login(api)?;
            config.access_token = Some(access_token);
            config.save()?;

            println!("Login successfully");
        }
        cli::Command::Course { id } => {
            let courses = codeptit::course::fetch(api)?;
            if let Some(id) = id {
                if let Some(course) = courses.into_iter().find(|course| course.id == id) {
                    config.course_id = Some(course.id);
                    config.save()?;

                    println!(
                        "Course ID {}: {} - {} selected",
                        course.id, course.subject.code, course.subject.name
                    );
                } else {
                    eprintln!("No course with ID {id} found");
                }
            } else {
                let mut builder = tabled::builder::Builder::new();
                builder.push_record(["ID", "Subject", "Semester"]);

                for course in courses {
                    builder.push_record([
                        format!(
                            "{}{}",
                            course.id.to_string(),
                            config
                                .course_id
                                .is_some_and(|id| course.id == id)
                                .then_some("*")
                                .unwrap_or_default()
                        ),
                        format!("{} - {}", course.subject.code, course.subject.name),
                        course.semester.name,
                    ]);
                }

                let mut table = builder.build();
                table.with(tabled::settings::Style::rounded());
                println!("{table}");
            }
        }
    };
    Ok(())
}
