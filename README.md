# RenCodeX — Tauri 2 + Svelte 5 + Rust

Encodeur vidéo desktop H265/HEVC NVENC pour Windows 11.

## Prérequis

| Outil | Version minimale |
|-------|-----------------|
| [Rust + Cargo](https://rustup.rs/) | 1.77+ (stable) |
| [Node.js](https://nodejs.org/) | 18+ |
| [FFmpeg + ffprobe](https://ffmpeg.org/) | 6.x (avec NVENC) |
| GPU NVIDIA | Kepler+ (GTX 600+) |
| Windows | 10/11 (x64) |

### Installer les pré-requis Tauri (Windows)
```powershell
# Outils C++ (WebView2 est inclus dans Win11)
winget install Microsoft.VisualStudio.2022.BuildTools
# Choisir "Desktop development with C++"
```

## Installation

```powershell
# 1. Cloner / extraire le projet
cd rencodex

# 2. Installer les dépendances JS
npm install

# 3. Configurer le chemin FFmpeg
# Éditer src-tauri/src/lib.rs ligne :
#   fn ffmpeg_path() -> PathBuf { ... }
# OU déposer ffmpeg.exe dans C:\Outil\ffmpeg\bin\ffmpeg.exe (chemin par défaut)
```

## Développement

```powershell
npm run tauri dev
```

## Build production

```powershell
npm run tauri build
# L'installateur .msi / .exe sera dans :
# src-tauri/target/release/bundle/
```

## Architecture

```
rencodex/
├── src-tauri/           # Backend Rust
│   ├── src/
│   │   ├── main.rs      # Point d'entrée Tauri
│   │   └── lib.rs       # Toute la logique : ffprobe, encodage, email
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                 # Frontend Svelte 5
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +layout.ts
│   │   └── +page.svelte  # Page principale
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
│   │   └── utils.ts               # Formatage temps / taille
│   └── app.css                    # Variables CSS thème
│
├── package.json
├── svelte.config.js
├── tailwind.config.js
└── vite.config.ts
```

## Fonctionnalités portées depuis Python

| Feature | Statut |
|---------|--------|
| Drag & drop fichiers | ✅ |
| Analyse ffprobe async | ✅ |
| Détection langues audio/sous-titres | ✅ |
| Sélection pistes à conserver | ✅ |
| Encodage H265 NVENC (cq=28, main10, p010le) | ✅ |
| Progression temps-réel | ✅ |
| Calcul vitesse / temps restant | ✅ |
| Nettoyage nom de fichier (regex) | ✅ |
| Renommage double-clic | ✅ |
| Annulation propre | ✅ |
| Dark / Light mode | ✅ |
| Config persistante JSON | ✅ |
| Rapport email (SMTP) | ✅ |

## Notes importantes

- **NVENC uniquement** : codec `hevc_nvenc`. Si votre GPU ne supporte pas NVENC, 
  remplacez dans `lib.rs` par `libx265` (encodage CPU, plus lent).
- **Pause/Reprise** : la version Python pausait FFmpeg via CTRL_C puis reprenait 
  avec `-ss`. Tauri ne supporte pas SIGSTOP sur Windows proprement, 
  donc la pause annule + redémarre. Un vrai pause peut être implémenté 
  via `CREATE_SUSPENDED` si nécessaire.
- **Email** : configurez vos identifiants dans les Settings (à ajouter à la page Settings).

## Personnalisation FFmpeg

Dans `src-tauri/src/lib.rs`, la fonction `start_encoding` construit la commande.
Pour changer les paramètres d'encodage, modifiez :
```rust
"-cq".into(), "28".into(),      // Qualité constante (18-35, plus bas = meilleure qualité)
"-preset".into(), "p5".into(),  // Vitesse encodage NVENC (p1=lent/qualité, p7=rapide)
```
