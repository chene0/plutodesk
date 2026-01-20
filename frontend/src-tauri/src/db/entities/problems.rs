use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "problems")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub set_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub s3_image_key: Option<String>,
    pub confidence_level: i32,
    pub notes: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub last_attempted: Option<DateTime>,
    pub attempt_count: i32,
    pub success_rate: f32,
    pub is_synced: bool,
    pub last_modified: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sets::Entity",
        from = "Column::SetId",
        to = "super::sets::Column::Id"
    )]
    Set,
    #[sea_orm(has_many = "super::problem_attempts::Entity")]
    ProblemAttempts,
}

impl Related<super::sets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Set.def()
    }
}

impl Related<super::problem_attempts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProblemAttempts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
