//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "bytecode_parts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub bytecode_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub order: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub part_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bytecodes::Entity",
        from = "Column::BytecodeId",
        to = "super::bytecodes::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Bytecodes,
    #[sea_orm(
        belongs_to = "super::parts::Entity",
        from = "Column::PartId",
        to = "super::parts::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Parts,
}

impl Related<super::bytecodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bytecodes.def()
    }
}

impl Related<super::parts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}