//! Catalogue des champs Discord disponibles par type de notification.

use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct FieldDef {
    pub id:    &'static str,
    pub label: &'static str,
}

/// Retourne le catalogue sous forme de HashMap<notifType, Vec<FieldDef>>
/// pour que le frontend puisse faire catalog["summary"], catalog["start"], etc.
pub fn full_catalog() -> HashMap<&'static str, Vec<FieldDef>> {
    let mut map = HashMap::new();

    map.insert("summary", vec![
        FieldDef { id: "files",    label: "Fichiers traités" },
        FieldDef { id: "gain",     label: "Gain total (%)" },
        FieldDef { id: "duration", label: "Durée totale" },
        FieldDef { id: "saved",    label: "Espace libéré" },
        FieldDef { id: "detail",   label: "Détail par fichier" },
    ]);

    map.insert("start", vec![
        FieldDef { id: "files",  label: "Nombre de fichiers" },
        FieldDef { id: "size",   label: "Taille totale" },
        FieldDef { id: "crf",    label: "CRF" },
        FieldDef { id: "preset", label: "Preset" },
    ]);

    map.insert("file_done", vec![
        FieldDef { id: "size_before", label: "Taille avant" },
        FieldDef { id: "size_after",  label: "Taille après" },
        FieldDef { id: "gain",        label: "Gain (%)" },
        FieldDef { id: "duration",    label: "Durée d'encodage" },
        FieldDef { id: "crf",         label: "CRF" },
        FieldDef { id: "preset",      label: "Preset" },
    ]);

    map.insert("error", vec![
        FieldDef { id: "file",    label: "Fichier" },
        FieldDef { id: "message", label: "Message d'erreur" },
    ]);

    map.insert("progress", vec![
        FieldDef { id: "file",      label: "Fichier en cours" },
        FieldDef { id: "index",     label: "Position dans la file" },
        FieldDef { id: "percent",   label: "Progression (%)" },
        FieldDef { id: "speed",     label: "Vitesse (x)" },
        FieldDef { id: "remaining", label: "Temps restant" },
        FieldDef { id: "elapsed",   label: "Temps écoulé" },
    ]);

    map
}

/// Retourne les champs activés par défaut pour un type de notification donné.
pub fn default_fields(notif_type: &str) -> Vec<String> {
    match notif_type {
        "summary"   => vec!["files", "gain", "duration", "saved", "detail"],
        "start"     => vec!["files", "size", "crf", "preset"],
        "file_done" => vec!["size_before", "size_after", "gain", "duration"],
        "error"     => vec!["file", "message"],
        "progress"  => vec!["file", "index", "percent", "speed", "remaining"],
        _           => vec![],
    }
    .into_iter().map(String::from).collect()
}