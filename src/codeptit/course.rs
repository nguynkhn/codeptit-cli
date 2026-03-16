// #[derive(serde::Deserialize)]
// pub struct CourseSemester {
//     pub name: String,
// }

#[derive(serde::Deserialize)]
pub struct CourseSubject {
    pub code: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct Course {
    pub id: crate::codeptit::api::ApiId,
    // pub semester: CourseSemester,
    pub subject: CourseSubject,
}

#[derive(serde::Deserialize)]
pub struct CourseResponse {
    #[serde(rename = "data")]
    pub courses: Vec<Course>,
}

impl<'a> crate::codeptit::api::Api<'a> {
    pub fn courses(&self) -> anyhow::Result<CourseResponse> {
        let response: CourseResponse = self
            .request(reqwest::Method::GET, "/courses/studying")
            .send()?
            .json()?;

        Ok(response)
    }
}
