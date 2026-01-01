pub struct ScreenshotDto {
    pub folder_name: String,
    pub course_name: String,
    pub subject_name: String,
    pub problem_name: String, // problem_name is equivalent to the screenshot name
    pub base64_data: String,
}
