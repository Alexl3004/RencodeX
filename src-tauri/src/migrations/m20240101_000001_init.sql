-- Compteurs globaux (toujours une seule ligne, id = 1)
CREATE TABLE IF NOT EXISTS stats (
    id                      INTEGER PRIMARY KEY,
    total_files             INTEGER NOT NULL DEFAULT 0,
    total_launched          INTEGER NOT NULL DEFAULT 0,
    total_original_mb       REAL    NOT NULL DEFAULT 0,
    total_encoded_mb        REAL    NOT NULL DEFAULT 0,
    sum_ratio_pct           REAL    NOT NULL DEFAULT 0,
    total_secs              REAL    NOT NULL DEFAULT 0,
    total_extracted_files   INTEGER NOT NULL DEFAULT 0,
    total_extract_launched  INTEGER NOT NULL DEFAULT 0,
    total_tracks_extracted  INTEGER NOT NULL DEFAULT 0,
    last_updated            TEXT
);

-- Sessions d'encodage
CREATE TABLE IF NOT EXISTS encode_sessions (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    date      TEXT    NOT NULL,
    files     INTEGER NOT NULL DEFAULT 0,
    ratio_pct REAL    NOT NULL DEFAULT 0,
    saved_mb  REAL    NOT NULL DEFAULT 0
);

-- Sessions d'extraction de sous-titres
CREATE TABLE IF NOT EXISTS extract_sessions (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    date   TEXT    NOT NULL,
    files  INTEGER NOT NULL DEFAULT 0,
    tracks INTEGER NOT NULL DEFAULT 0
);

-- Records (heaviest / best_ratio)
CREATE TABLE IF NOT EXISTS file_records (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    kind        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    original_mb REAL NOT NULL DEFAULT 0,
    encoded_mb  REAL NOT NULL DEFAULT 0,
    ratio_pct   REAL NOT NULL DEFAULT 0
);

-- Historique détaillé de chaque fichier encodé
CREATE TABLE IF NOT EXISTS encoded_files (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER REFERENCES encode_sessions(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    original_mb REAL NOT NULL DEFAULT 0,
    encoded_mb  REAL NOT NULL DEFAULT 0,
    ratio_pct   REAL NOT NULL DEFAULT 0,
    duration_s  REAL NOT NULL DEFAULT 0,
    encoded_at  TEXT NOT NULL
);

-- Ligne stats par défaut
INSERT OR IGNORE INTO stats (id) VALUES (1);