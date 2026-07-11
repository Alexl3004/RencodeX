//! Module de nommage : analyse et nettoyage des noms de fichiers vidéo.
//!
//! - `regex`    : expressions régulières pré-compilées (patterns uniquement)
//! - `filename` : pipeline de détection et construction du nom suggéré

pub mod regex;
pub mod filename;

// Re-export de l'entrée publique principale
pub use filename::clean_filename;