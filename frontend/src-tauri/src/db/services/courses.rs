use sea_orm::*;
use crate::db::entities::{courses, courses::Entity as Course};
use uuid::Uuid;

pub async fn create_course(
    db: &DatabaseConnection,
    folder_id: Uuid,
    name: String,
    description: Option<String>,
    color_code: Option<String>,
    sort_order: i32,
) -> Result<courses::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let course = courses::ActiveModel {
        id: Set(Uuid::new_v4()),
        folder_id: Set(folder_id),
        name: Set(name),
        description: Set(description),
        color_code: Set(color_code),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    course.insert(db).await
}

pub async fn get_course_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<courses::Model>, DbErr> {
    Course::find_by_id(id).one(db).await
}

pub async fn get_courses_by_folder(
    db: &DatabaseConnection,
    folder_id: Uuid,
) -> Result<Vec<courses::Model>, DbErr> {
    Course::find()
        .filter(courses::Column::FolderId.eq(folder_id))
        .order_by_asc(courses::Column::SortOrder)
        .all(db)
        .await
}

pub async fn update_course(
    db: &DatabaseConnection,
    id: Uuid,
    name: Option<String>,
    description: Option<Option<String>>,
    color_code: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<courses::Model, DbErr> {
    let course = Course::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Course not found".to_string()))?;

    let mut course: courses::ActiveModel = course.into();

    if let Some(n) = name {
        course.name = Set(n);
    }
    if let Some(d) = description {
        course.description = Set(d);
    }
    if let Some(cc) = color_code {
        course.color_code = Set(cc);
    }
    if let Some(so) = sort_order {
        course.sort_order = Set(so);
    }

    course.updated_at = Set(chrono::Utc::now().naive_utc());
    course.is_synced = Set(false);

    course.update(db).await
}

pub async fn delete_course(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    Course::delete_by_id(id).exec(db).await
}
