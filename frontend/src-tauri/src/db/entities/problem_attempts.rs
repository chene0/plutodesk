use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "problem_attempts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub problem_id: Uuid,
    pub time_spent_seconds: i32,
    pub difficulty_rating: i32,
    pub confidence_level: i32,
    pub was_successful: bool,
    pub notes: Option<String>,
    pub attempted_at: DateTime,
    pub is_synced: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::problems::Entity",
        from = "Column::ProblemId",
        to = "super::problems::Column::Id"
    )]
    Problem,
}

impl Related<super::problems::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Problem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
