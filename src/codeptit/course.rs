#[derive(serde::Deserialize)]
pub struct Semester {
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct Subject {
    pub code: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct Course {
    pub id: u32,
    pub semester: Semester,
    pub subject: Subject,
}

#[derive(serde::Deserialize)]
struct CourseResponse {
    data: Vec<Course>,
}

pub fn fetch(api: crate::codeptit::api::Api) -> anyhow::Result<Vec<Course>> {
    let response: CourseResponse = api
        .request(reqwest::Method::GET, "/courses/studying")
        .send()?
        .json()?;

    Ok(response.data)
}
