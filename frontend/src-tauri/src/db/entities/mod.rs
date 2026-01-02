pub mod users;
pub mod subscriptions;
pub mod folders;
pub mod courses;
pub mod subjects;
pub mod problems;
pub mod problem_attempts;

pub use users::Entity as Users;
pub use subscriptions::Entity as Subscriptions;
pub use folders::Entity as Folders;
pub use courses::Entity as Courses;
pub use subjects::Entity as Subjects;
pub use problems::Entity as Problems;
pub use problem_attempts::Entity as ProblemAttempts;
