use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "courses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub folder_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color_code: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub is_synced: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::folders::Entity",
        from = "Column::FolderId",
        to = "super::folders::Column::Id"
    )]
    Folder,
    #[sea_orm(has_many = "super::sets::Entity")]
    Sets,
}

impl Related<super::folders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Folder.def()
    }
}

impl Related<super::sets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
