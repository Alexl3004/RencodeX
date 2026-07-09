use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub total_files: i32,
    pub total_launched: i32,
    pub total_original_mb: f64,
    pub total_encoded_mb: f64,
    pub sum_ratio_pct: f64,
    pub total_secs: f64,
    pub total_extracted_files: i32,
    pub total_extract_launched: i32,
    pub total_tracks_extracted: i32,
    pub last_updated: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
