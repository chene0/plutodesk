use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subjects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub course_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub is_synced: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::courses::Entity",
        from = "Column::CourseId",
        to = "super::courses::Column::Id"
    )]
    Course,
    #[sea_orm(has_many = "super::problems::Entity")]
    Problems,
}

impl Related<super::courses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Course.def()
    }
}

impl Related<super::problems::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Problems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
