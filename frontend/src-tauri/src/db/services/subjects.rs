use sea_orm::*;
use crate::db::entities::{subjects, subjects::Entity as Subject};
use uuid::Uuid;

pub async fn create_subject(
    db: &DatabaseConnection,
    course_id: Uuid,
    name: String,
    description: Option<String>,
    sort_order: i32,
) -> Result<subjects::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let subject = subjects::ActiveModel {
        id: Set(Uuid::new_v4()),
        course_id: Set(course_id),
        name: Set(name),
        description: Set(description),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    subject.insert(db).await
}

pub async fn get_subject_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<subjects::Model>, DbErr> {
    Subject::find_by_id(id).one(db).await
}

pub async fn get_subjects_by_course(
    db: &DatabaseConnection,
    course_id: Uuid,
) -> Result<Vec<subjects::Model>, DbErr> {
    Subject::find()
        .filter(subjects::Column::CourseId.eq(course_id))
        .order_by_asc(subjects::Column::SortOrder)
        .all(db)
        .await
}

pub async fn update_subject(
    db: &DatabaseConnection,
    id: Uuid,
    name: Option<String>,
    description: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<subjects::Model, DbErr> {
    let subject = Subject::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Subject not found".to_string()))?;

    let mut subject: subjects::ActiveModel = subject.into();

    if let Some(n) = name {
        subject.name = Set(n);
    }
    if let Some(d) = description {
        subject.description = Set(d);
    }
    if let Some(so) = sort_order {
        subject.sort_order = Set(so);
    }

    subject.updated_at = Set(chrono::Utc::now().naive_utc());
    subject.is_synced = Set(false);

    subject.update(db).await
}

pub async fn delete_subject(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    Subject::delete_by_id(id).exec(db).await
}
