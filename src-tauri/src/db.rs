//! Couche de persistance SQLite pour RenCodeX — SeaORM.
//!
//! # Usage
//! Appeler `db::init().await` dans `main.rs` **avant** de construire le builder Tauri.
//! Toutes les commandes accèdent ensuite à la base via `crate::db::DB`.

use std::path::PathBuf;
use once_cell::sync::OnceCell;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::entities::{
    encode_sessions, extract_sessions, file_records, stats,
};
use crate::models::{EncodeSession, ExtractSession, FileRecord, Stats};

// ── Handle global ─────────────────────────────────────────────────────────────

/// Handle global vers la base SeaORM. Initialisé une seule fois via `init()`.
pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// Raccourci pour obtenir la connexion globale (panic si init() non appelé).
pub fn conn() -> &'static DatabaseConnection {
    DB.get().expect("db::init() non appelé avant utilisation de DB")
}

// ── Chemins ───────────────────────────────────────────────────────────────────

fn db_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RenCodeX")
        .join("data.db")
}

fn stats_json_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RenCodeX")
        .join("stats.json")
}

// ── Initialisation ────────────────────────────────────────────────────────────

/// Point d'entrée appelé depuis `main.rs`.
/// Ouvre la base, applique les migrations DDL, puis migre depuis stats.json si besoin.
pub async fn init() {
    let path = db_path();
    std::fs::create_dir_all(path.parent().unwrap()).ok();

    let url = format!("sqlite://{}?mode=rwc", path.display());

    let db = Database::connect(&url)
        .await
        .expect("Impossible d'ouvrir la base de données SQLite");

    // Migrations DDL via fichier SQL embarqué.
    let sql = include_str!("migrations/m20240101_000001_init.sql");
    for statement in sql.split(';').map(str::trim).filter(|s: &&str| !s.is_empty()) {
        sea_orm::ConnectionTrait::execute_unprepared(&db, statement)
            .await
            .expect("Erreur lors de la migration DDL");
    }

    DB.set(db).expect("DB déjà initialisé");

    migrate_from_json().await;
}

// ── Migration JSON → SQLite ───────────────────────────────────────────────────

async fn migrate_from_json() {
    let json_path = stats_json_path();
    if !json_path.exists() {
        return;
    }

    // Ne migre que si la base est vierge.
    let current: Option<stats::Model> = stats::Entity::find_by_id(1)
        .one(conn())
        .await
        .unwrap_or(None);

    if let Some(s) = &current {
        if s.total_files > 0 {
            return;
        }
    }

    let data = match std::fs::read_to_string(&json_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[db] Impossible de lire stats.json : {}", e);
            return;
        }
    };

    let s: crate::models::Stats = match serde_json::from_str(&data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[db] Erreur parsing stats.json : {}", e);
            return;
        }
    };

    // Mise à jour des compteurs globaux.
    let active = stats::ActiveModel {
        id: Set(1_i32),
        total_files: Set(s.total_files as i32),
        total_launched: Set(s.total_launched as i32),
        total_original_mb: Set(s.total_original_mb),
        total_encoded_mb: Set(s.total_encoded_mb),
        sum_ratio_pct: Set(s.sum_ratio_pct),
        total_secs: Set(s.total_secs),
        total_extracted_files: Set(s.total_extracted_files as i32),
        total_extract_launched: Set(s.total_extract_launched as i32),
        total_tracks_extracted: Set(s.total_tracks_extracted as i32),
        last_updated: Set(s.last_updated.clone()),
    };
    if let Err(e) = active.update(conn()).await {
        eprintln!("[db] Erreur migration compteurs : {}", e);
        return;
    }

    // Sessions d'encodage (ordre chronologique).
    for session in s.encode_sessions.iter().rev() {
        let _ = encode_sessions::ActiveModel {
            date: Set(session.date.clone()),
            files: Set(session.files as i32),
            ratio_pct: Set(session.ratio_pct),
            saved_mb: Set(session.saved_mb),
            ..Default::default()
        }
        .insert(conn())
        .await;
    }

    // Sessions d'extraction.
    for session in s.extract_sessions.iter().rev() {
        let _ = extract_sessions::ActiveModel {
            date: Set(session.date.clone()),
            files: Set(session.files as i32),
            tracks: Set(session.tracks as i32),
            ..Default::default()
        }
        .insert(conn())
        .await;
    }

    // Records.
    if let Some(r) = &s.record_heaviest {
        let _ = upsert_record("heaviest", r).await;
    }
    if let Some(r) = &s.record_best_ratio {
        let _ = upsert_record("best_ratio", r).await;
    }

    // Renommer stats.json en .bak.
    let bak = json_path.with_extension("json.bak");
    match std::fs::rename(&json_path, &bak) {
        Ok(_) => eprintln!("[db] Migration stats.json → SQLite réussie. Sauvegarde : {:?}", bak),
        Err(e) => eprintln!("[db] Impossible de renommer stats.json : {}", e),
    }
}

// ── Helpers publics ───────────────────────────────────────────────────────────

/// Charge les statistiques globales depuis SQLite.
pub async fn load_stats_from_db() -> Result<Stats, sea_orm::DbErr> {
    // Compteurs globaux.
    let row: stats::Model = stats::Entity::find_by_id(1)
        .one(conn())
        .await?
        .expect("Ligne stats manquante (id=1)");

    // 10 dernières sessions d'encodage.
    let encode_rows: Vec<encode_sessions::Model> = encode_sessions::Entity::find()
        .order_by_desc(encode_sessions::Column::Id)
        .limit(10)
        .all(conn())
        .await?;

    // 10 dernières sessions d'extraction.
    let extract_rows: Vec<extract_sessions::Model> = extract_sessions::Entity::find()
        .order_by_desc(extract_sessions::Column::Id)
        .limit(10)
        .all(conn())
        .await?;

    // Records.
    let record_heaviest   = load_record("heaviest").await?;
    let record_best_ratio = load_record("best_ratio").await?;

    Ok(Stats {
        total_files:            row.total_files as u32,
        total_launched:         row.total_launched as u32,
        total_original_mb:      row.total_original_mb,
        total_encoded_mb:       row.total_encoded_mb,
        sum_ratio_pct:          row.sum_ratio_pct,
        total_secs:             row.total_secs,
        total_extracted_files:  row.total_extracted_files as u32,
        total_extract_launched: row.total_extract_launched as u32,
        total_tracks_extracted: row.total_tracks_extracted as u32,
        last_updated:           row.last_updated,
        encode_sessions: encode_rows
            .into_iter()
            .map(|s| EncodeSession {
                date:      s.date,
                files:     s.files as u32,
                ratio_pct: s.ratio_pct,
                saved_mb:  s.saved_mb,
            })
            .collect(),
        extract_sessions: extract_rows
            .into_iter()
            .map(|s| ExtractSession {
                date:   s.date,
                files:  s.files as u32,
                tracks: s.tracks as u32,
            })
            .collect(),
        record_heaviest,
        record_best_ratio,
    })
}

/// Persiste les statistiques globales dans SQLite.
pub async fn save_stats_to_db(s: &Stats) -> Result<(), sea_orm::DbErr> {
    // Mise à jour des compteurs globaux.
    stats::ActiveModel {
        id: Set(1_i32),
        total_files: Set(s.total_files as i32),
        total_launched: Set(s.total_launched as i32),
        total_original_mb: Set(s.total_original_mb),
        total_encoded_mb: Set(s.total_encoded_mb),
        sum_ratio_pct: Set(s.sum_ratio_pct),
        total_secs: Set(s.total_secs),
        total_extracted_files: Set(s.total_extracted_files as i32),
        total_extract_launched: Set(s.total_extract_launched as i32),
        total_tracks_extracted: Set(s.total_tracks_extracted as i32),
        last_updated: Set(s.last_updated.clone()),
    }
    .update(conn())
    .await?;

    // Insérer la session d'encodage la plus récente si nouvelle.
    if let Some(session) = s.encode_sessions.first() {
        let exists = encode_sessions::Entity::find()
            .filter(encode_sessions::Column::Date.eq(&session.date))
            .count(conn())
            .await?;

        if exists == 0 {
            encode_sessions::ActiveModel {
                date: Set(session.date.clone()),
                files: Set(session.files as i32),
                ratio_pct: Set(session.ratio_pct),
                saved_mb: Set(session.saved_mb),
                ..Default::default()
            }
            .insert(conn())
            .await?;
        }
    }

    // Insérer la session d'extraction la plus récente si nouvelle.
    if let Some(session) = s.extract_sessions.first() {
        let exists = extract_sessions::Entity::find()
            .filter(extract_sessions::Column::Date.eq(&session.date))
            .count(conn())
            .await?;

        if exists == 0 {
            extract_sessions::ActiveModel {
                date: Set(session.date.clone()),
                files: Set(session.files as i32),
                tracks: Set(session.tracks as i32),
                ..Default::default()
            }
            .insert(conn())
            .await?;
        }
    }

    // Upsert des records.
    if let Some(r) = &s.record_heaviest {
        upsert_record("heaviest", r).await?;
    }
    if let Some(r) = &s.record_best_ratio {
        upsert_record("best_ratio", r).await?;
    }

    Ok(())
}

// ── Helpers privés ────────────────────────────────────────────────────────────

async fn load_record(kind: &str) -> Result<Option<FileRecord>, sea_orm::DbErr> {
    let row: Option<file_records::Model> = file_records::Entity::find()
        .filter(file_records::Column::Kind.eq(kind))
        .one(conn())
        .await?;

    Ok(row.map(|r| FileRecord {
        name:        r.name,
        original_mb: r.original_mb,
        encoded_mb:  r.encoded_mb,
        ratio_pct:   r.ratio_pct,
    }))
}

async fn upsert_record(kind: &str, r: &FileRecord) -> Result<(), sea_orm::DbErr> {
    // Cherche si le record existe déjà.
    let existing = file_records::Entity::find()
        .filter(file_records::Column::Kind.eq(kind))
        .one(conn())
        .await?;

    match existing {
        Some(row) => {
            // Mise à jour.
            file_records::ActiveModel {
                id: Set(row.id),
                kind: Set(kind.to_string()),
                name: Set(r.name.clone()),
                original_mb: Set(r.original_mb),
                encoded_mb: Set(r.encoded_mb),
                ratio_pct: Set(r.ratio_pct),
            }
            .update(conn())
            .await?;
        }
        None => {
            // Insertion.
            file_records::ActiveModel {
                kind: Set(kind.to_string()),
                name: Set(r.name.clone()),
                original_mb: Set(r.original_mb),
                encoded_mb: Set(r.encoded_mb),
                ratio_pct: Set(r.ratio_pct),
                ..Default::default()
            }
            .insert(conn())
            .await?;
        }
    }

    Ok(())
}