use sea_orm::*;
use crate::db::entities::{sets, sets::Entity as SetEntity};
use uuid::Uuid;

pub async fn create_set(
    db: &DatabaseConnection,
    course_id: Uuid,
    name: String,
    description: Option<String>,
    sort_order: i32,
) -> Result<sets::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let set = sets::ActiveModel {
        id: Set(Uuid::new_v4()),
        course_id: Set(course_id),
        name: Set(name),
        description: Set(description),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    set.insert(db).await
}

pub async fn get_set_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<sets::Model>, DbErr> {
    SetEntity::find_by_id(id).one(db).await
}

pub async fn get_sets_by_course(
    db: &DatabaseConnection,
    course_id: Uuid,
) -> Result<Vec<sets::Model>, DbErr> {
    SetEntity::find()
        .filter(sets::Column::CourseId.eq(course_id))
        .order_by_asc(sets::Column::SortOrder)
        .all(db)
        .await
}

pub async fn update_set(
    db: &DatabaseConnection,
    id: Uuid,
    name: Option<String>,
    description: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<sets::Model, DbErr> {
    let set = SetEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Set not found".to_string()))?;

    let mut set: sets::ActiveModel = set.into();

    if let Some(n) = name {
        set.name = Set(n);
    }
    if let Some(d) = description {
        set.description = Set(d);
    }
    if let Some(so) = sort_order {
        set.sort_order = Set(so);
    }

    set.updated_at = Set(chrono::Utc::now().naive_utc());
    set.is_synced = Set(false);

    set.update(db).await
}

pub async fn delete_set(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    SetEntity::delete_by_id(id).exec(db).await
}
