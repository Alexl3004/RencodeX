# RenCodeX — Tauri 2 + Svelte 5 + Rust

Encodeur vidéo desktop **H265/HEVC NVENC** pour Windows, avec nettoyage intelligent de noms de fichiers, notifications Discord, rapports email et scan de dossiers.

---

## Prérequis

| Outil | Version minimale |
|---|---|
| [Rust + Cargo](https://rustup.rs/) | 1.77+ (stable) |
| [Node.js](https://nodejs.org/) | 18+ |
| [FFmpeg + ffprobe](https://ffmpeg.org/) | 6.x (avec NVENC) |
| GPU NVIDIA | Kepler+ (GTX 600+) |
| Windows | 10/11 (x64) |

### Outils C++ requis par Tauri

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
# Cocher "Desktop development with C++" dans l'installateur
# WebView2 est inclus nativement dans Windows 11
```

---

## Installation

```powershell
# 1. Cloner le dépôt
git clone https://github.com/Alexl3004/RencodeX.git
cd RencodeX

# 2. Installer les dépendances JS
npm install
```

### Configurer le chemin FFmpeg

Par défaut, RenCodeX cherche FFmpeg ici :

```
C:\Outil\ffmpeg\bin\ffmpeg.exe
```

Pour utiliser un chemin différent, deux options :

**Option A — Variable d'environnement (recommandée) :**
```powershell
$env:RENCODEX_FFMPEG_PATH = "C:\chemin\vers\ffmpeg.exe"
```

**Option B — Via l'interface :** modifier le chemin dans les Settings de l'application (sauvegardé dans `%APPDATA%\RenCodeX\config.json`).

---

## Développement

```powershell
npm run tauri dev
```

## Build production

```powershell
npm run tauri build
# Installateur généré dans :
# src-tauri/target/release/bundle/   (.msi et .exe)
```

---

## Architecture

```
RenCodeX/
├── src-tauri/                   # Backend Rust
│   ├── src/
│   │   ├── main.rs              # Point d'entrée Tauri + démarrage bot Discord
│   │   ├── models.rs            # Structs partagées (AppConfig, EncodeJob, etc.)
│   │   ├── state.rs             # État global de l'encodeur (Mutex partagé)
│   │   ├── regex.rs             # Regex pré-compilées (Once_cell) pour noms de fichiers
│   │   ├── utils.rs             # Utilitaires : chemins, langues, config, resolve_config
│   │   ├── filename.rs          # Logique de nettoyage et parsing de noms de fichiers
│   │   ├── media.rs             # Analyse ffprobe + encodage FFmpeg async
│   │   ├── notify.rs            # Notifications Discord (bot + webhooks) + email SMTP
│   │   └── commands.rs          # Commandes Tauri exposées au frontend
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                         # Frontend Svelte 5
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +layout.ts
│   │   └── +page.svelte         # Page principale
│   ├── components/
│   │   ├── TopBar.svelte
│   │   ├── DropZone.svelte
│   │   ├── FileTable.svelte
│   │   ├── LangSelector.svelte
│   │   ├── ProgressPanel.svelte
│   │   ├── LogConsole.svelte
│   │   ├── OutputDirPicker.svelte
│   │   └── ControlBar.svelte
│   ├── lib/
│   │   ├── stores/
│   │   │   ├── encoder.svelte.ts   # Store Svelte 5 runes (état encodeur)
│   │   │   └── theme.svelte.ts     # Store thème dark/light
│   │   └── utils.ts               # Formatage durée / taille
│   └── app.css                    # Variables CSS thème
│
├── package.json
├── svelte.config.js
├── vite.config.ts
└── .gitignore
```

---

## Fonctionnalités

| Fonctionnalité | Statut |
|---|---|
| Drag & drop fichiers | ✅ |
| Scan de dossier récursif | ✅ |
| Analyse ffprobe async (streams, langues, durée, FPS) | ✅ |
| Détection langues audio & sous-titres | ✅ |
| Sélection des pistes à conserver | ✅ |
| Encodage H265 NVENC (cq=28, main10, p010le) | ✅ |
| Progression temps-réel (vitesse, temps restant) | ✅ |
| Annulation propre (suppression du fichier partiel) | ✅ |
| Nettoyage intelligent de noms de fichiers (regex) | ✅ |
| Renommage double-clic dans la file | ✅ |
| Dark / Light mode | ✅ |
| Config persistante JSON (`%APPDATA%\RenCodeX\`) | ✅ |
| Notifications Discord (bot + embeds rich) | ✅ |
| Bot Discord avec commandes à distance | ✅ |
| Rapport email (SMTP / lettre) | ✅ |

---

## Configuration Discord

RenCodeX peut envoyer des notifications Discord via un bot et écouter des commandes à distance.

**Variables d'environnement (prioritaires sur la config UI) :**

```powershell
$env:RENCODEX_DISCORD_TOKEN = "Bot_token_ici"
```

**Via l'UI Settings :**
- `discord_bot_token` — token du bot Discord
- `discord_log_channel_id` — channel pour les notifications (résumés, erreurs, progression)
- `discord_cmd_channel_id` — channel pour les commandes à distance (bot en écoute)

**Notifications disponibles :**
- Début d'encodage (total fichiers, taille, paramètres)
- Fin de chaque fichier (taille avant/après, durée)
- Erreurs d'encodage
- Progression périodique (intervalle configurable, désactivé par défaut)
- Résumé final de session

---

## Configuration Email

Le rapport email est envoyé via SMTP à la fin d'une session d'encodage.

Les identifiants sont passés depuis le frontend lors de l'appel à `send_email_report` (non stockés en clair dans la config principale) :

```json
{
  "smtp_host": "smtp.gmail.com",
  "smtp_port": 587,
  "username": "ton@email.com",
  "password": "app_password",
  "to": "destinataire@email.com"
}
```

---

## Personnalisation de l'encodage

Dans `src-tauri/src/media.rs`, la fonction `start_encoding` construit la commande FFmpeg. Paramètres clés :

```rust
"-cq".into(), "28".into(),     // Qualité (18–35, plus bas = meilleure qualité)
"-preset".into(), "p5".into(), // Vitesse NVENC (p1=lent/qualité ↔ p7=rapide)
```

Pour encoder en CPU (sans GPU NVIDIA), remplacer `hevc_nvenc` par `libx265` dans la commande FFmpeg.

---

## Notes

- **NVENC uniquement par défaut** : le codec utilisé est `hevc_nvenc`. Un GPU NVIDIA Kepler+ est requis. Pour un encodage CPU, substituer `libx265` dans `media.rs`.
- **Pause/Reprise** : non implémentée nativement (SIGSTOP non supporté proprement sur Windows). L'annulation supprime le fichier de sortie partiel. Une vraie pause via `CREATE_SUSPENDED` est envisageable.
- **Fichier de config** : stocké dans `%APPDATA%\RenCodeX\config.json`. Aucun credential sensible (token, mot de passe) n'est sauvegardé dans ce fichier — ils sont fournis à chaque session via l'UI ou des variables d'environnement.