# RenCodeX — Tauri 2 · Svelte 5 · Rust

Encodeur vidéo desktop **H265/HEVC NVENC** pour Windows, avec nettoyage intelligent de noms de fichiers entièrement personnalisable, notifications Discord riches, commandes bot à distance et scan de dossiers récursif.

---

## Prérequis

| Outil                                   | Version minimale   |
| --------------------------------------- | ------------------ |
| [Rust + Cargo](https://rustup.rs/)      | 1.77+ (stable)     |
| [Node.js](https://nodejs.org/)          | 18+                |
| [FFmpeg + ffprobe](https://ffmpeg.org/) | 6.x (avec NVENC)   |
| GPU NVIDIA                              | Kepler+ (GTX 600+) |
| Windows                                 | 10/11 (x64)        |

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

Par défaut, RenCodeX cherche FFmpeg à :

```
C:\Outil\ffmpeg\bin\ffmpeg.exe
```

Pour utiliser un chemin différent, deux options :

**Option A — Variable d'environnement (prioritaire) :**

```powershell
$env:RENCODEX_FFMPEG_PATH = "C:\chemin\vers\ffmpeg.exe"
```

**Option B — Via l'interface :** modifier le chemin dans Paramètres → onglet _FFmpeg_. La valeur est sauvegardée dans `%APPDATA%\RenCodeX\config.json`.

> Les variables d'environnement ont la priorité absolue sur la config UI pour le chemin FFmpeg. Pour Discord, c'est l'inverse : la config UI est prioritaire, les variables d'env ne servent que de fallback si le champ est vide.

---

## Développement

```powershell
npm run tauri dev
```

## Build production

```powershell
npm run tauri build
# Installateur généré dans :
# src-tauri/target/release/bundle/   (.msi et .exe NSIS)
```

---

## Architecture

```
RenCodeX/
├── src-tauri/                              # Backend Rust
│   ├── src/
│   │   ├── main.rs                         # Point d'entrée Tauri + supervision bot Discord (backoff exponentiel)
│   │   ├── lib.rs                          # Exports publics pour les tests d'intégration
│   │   ├── models.rs                       # Structs partagées (AppConfig, EncodingPrefs, EncodeJob…)
│   │   ├── state.rs                        # État global atomique de l'encodeur (flags cancel/pause/skip)
│   │   ├── db.rs                           # Couche SQLite via SeaORM (init, migrations, CRUD)
│   │   ├── utils.rs                        # Utilitaires : chemins FFmpeg, langues, title case, resolve_config
│   │   ├── commands/                       # Commandes Tauri exposées au frontend
│   │   │   ├── encoding.rs                 # check_ffmpeg, analyze_file, clean_filename, start/cancel/pause/resume/skip
│   │   │   ├── files.rs                    # get_default_output_dir, list_subtitle_tracks, extract_subtitles, scan_folder
│   │   │   ├── settings.rs                 # load/save config & prefs, stats, champs Discord, statut bot
│   │   │   └── discord.rs                  # send_discord_* (start, file_done, error, progress, stats)
│   │   ├── services/
│   │   │   ├── media.rs                    # Analyse ffprobe + encodage FFmpeg async + progression temps réel
│   │   │   ├── notify.rs                   # Notifications Discord (embeds)
│   │   │   ├── discord_bot.rs              # Bot Twilight (gateway WebSocket) — commandes à distance
│   │   │   └── discord_fields.rs           # Catalogue et valeurs par défaut des champs d'embed Discord
│   │   ├── naming/
│   │   │   ├── filename.rs                 # Logique de nettoyage et parsing des noms de fichiers
│   │   │   └── regex.rs                    # Regex pré-compilées (once_cell) + normalize_hdr_tags
│   │   ├── entities/                       # Entités SeaORM (stats, encode_sessions, extract_sessions, file_records)
│   │   └── migrations/
│   │       └── m20240101_000001_init.sql   # DDL SQLite initial
│   ├── tests/
│   │   ├── tests.rs                        # Tests unitaires (noms de fichiers, langues, title case)
│   │   └── tests_extended.rs               # Tests étendus (regex, cas limites)
│   ├── capabilities/
│   │   └── default.json                    # Permissions Tauri (fs, dialog, shell, notification)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                                    # Frontend Svelte 5 (runes)
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +layout.ts                      # SSR désactivé (adapter-static)
│   │   └── +page.svelte                    # Shell principal + navigation (vertical/horizontal)
│   ├── ui/                                 # Composants (alias $components)
│   │   ├── TopBar.svelte                   # Barre du haut : logo, statut encodeur, navigation
│   │   ├── FileToolbar.svelte              # Boutons ajout de fichiers / scan dossier + presets de dossier
│   │   ├── FileTable.svelte                # Table des fichiers en file d'attente + drag & drop natif Tauri
│   │   ├── ProgressPanel.svelte            # Barre de progression + vitesse moyennée + ETA fichier & total
│   │   ├── ControlBar.svelte               # Boutons Encoder / Pause / Reprendre / Annuler
│   │   ├── OutputDirPicker.svelte          # Sélecteur de dossier de sortie
│   │   ├── ToastNotif.svelte               # Notifications toast
│   │   ├── Log/
│   │   │   └── LogConsole.svelte           # Console de logs en temps réel
│   │   ├── modals/
│   │   │   ├── FileLangModal.svelte        # Sélection des pistes audio/sous-titres par fichier
│   │   │   ├── FileModal.svelte            # Détail d'un fichier (streams, infos, métadonnées HDR)
│   │   │   └── FileRenameModal.svelte      # Renommage assisté avec aperçu du nom généré
│   │   ├── popovers/
│   │   │   ├── LangPopover.svelte          # Popover de langue inline dans la table
│   │   │   └── LangSelector.svelte         # Sélecteur de langues global
│   │   ├── encoding/
│   │   │   ├── EncodingSettings.svelte     # Panneau principal (onglets vidéo / audio / sous-titres / conteneur / presets)
│   │   │   ├── VideoTab.svelte             # CRF, preset NVENC, AQ, multipass, mode vidéo, HDR/DV
│   │   │   ├── AudioTab.svelte             # Règles audio par codec source (copie / réencodage)
│   │   │   ├── SubtitlesTab.svelte         # Sélection et comportement des pistes de sous-titres
│   │   │   ├── ContainerTab.svelte         # Choix du conteneur (MKV / MP4)
│   │   │   ├── LanguagesTab.svelte         # Ordre des langues détectées
│   │   │   └── PresetsTab.svelte           # Presets d'encodage intégrés & personnalisés
│   │   ├── renaming/
│   │   │   ├── RenamingSettings.svelte     # Panneau de renommage (onglets format / tags / team / aperçu)
│   │   │   ├── FormatTab.svelte            # Format S/E, casse, séparateur, format provider
│   │   │   ├── TagsTab.svelte              # Ordre et activation des tags du nom de sortie
│   │   │   ├── TeamTab.svelte              # Tag team libre
│   │   │   └── PreviewTab.svelte           # Aperçu en temps réel du nom généré
│   │   ├── dashboard/
│   │   │   ├── Dashboard.svelte            # Shell du tableau de bord (onglets)
│   │   │   ├── EncodageTab.svelte          # Statistiques cumulées d'encodage
│   │   │   ├── ExtractionTab.svelte        # Statistiques d'extraction de sous-titres
│   │   │   └── HistoryTab.svelte           # Historique des sessions avec détail par fichier
│   │   └── setting/
│   │       ├── Settings.svelte             # Shell des paramètres (onglets)
│   │       ├── FfmpegTab.svelte            # Chemin FFmpeg + vérification
│   │       ├── DiscordTab.svelte           # Token bot, salons, activation des notifications
│   │       ├── BotTab.svelte               # Configuration avancée du bot et des embeds Discord
│   │       ├── NotifRow.svelte             # Ligne de configuration d'un type de notification
│   │       ├── InterfaceTab.svelte         # Thème, layout, palette de couleurs
│   │       ├── EnvTab.svelte               # Variables d'environnement détectées
│   │       └── PresetsTab.svelte           # Gestion des presets de dossiers de sortie
│   └── lib/
│       ├── stores/
│       │   ├── encoder.svelte.ts           # Store principal : fichiers en file, état encodage, navigation
│       │   ├── encoding.store.svelte.ts    # Logique d'encodage (résolution des règles audio, envoi des jobs)
│       │   ├── files.store.svelte.ts       # Gestion des fichiers (ajout, suppression, analyse ffprobe)
│       │   ├── prefs.store.svelte.ts       # Préférences d'encodage + presets intégrés & personnalisés
│       │   ├── naming.ts                   # Constantes de renommage (formats, séparateurs, ordre des tags)
│       │   ├── stats.svelte.ts             # Statistiques cumulées et sessions (lecture/écriture SQLite)
│       │   ├── theme.svelte.ts             # Thème dark/light + palettes de couleurs
│       │   ├── toasts.svelte.ts            # File de notifications toast
│       │   └── types.ts                    # Types partagés (StreamInfo, EncodeJob, AudioCodecRules…)
│       └── utils.ts                        # Formatage durée / taille
└── app.css                                 # Variables CSS globales (thème, radius, fonts)
```

---

## Fonctionnalités

| Fonctionnalité                                                       | Statut |
| -------------------------------------------------------------------- | ------ |
| Drag & drop fichiers (natif Tauri)                                   | ✅     |
| Ajout de fichiers par sélection multiple                             | ✅     |
| Scan de dossier récursif                                             | ✅     |
| Presets de dossiers de sortie                                        | ✅     |
| Analyse ffprobe async (streams, langues, durée, FPS, HDR)            | ✅     |
| Détection et sélection des pistes audio & sous-titres                | ✅     |
| Override de langue par piste (par fichier)                           | ✅     |
| Encodage H265 NVENC (CRF configurable, main10, p010le)               | ✅     |
| Mode vidéo copie (passe-through sans réencodage)                     | ✅     |
| Préservation des métadonnées HDR (color_primaries, color_transfer)   | ✅     |
| Support Dolby Vision (preserve_dv — MKV recommandé)                  | ✅     |
| AQ spatiale / temporelle + force de l'AQ configurable                | ✅     |
| Multipass NVENC (disabled / qres / fullres)                          | ✅     |
| Conteneur MKV ou MP4                                                 | ✅     |
| Règles audio par codec source (copie / réencodage par codec)         | ✅     |
| Modes audio prédéfinis : Tout copier / Intelligent / Tout réencoder  | ✅     |
| Presets d'encodage intégrés (Rapide / Équilibré / Qualité / Archive) | ✅     |
| Presets d'encodage personnalisés (sauvegarde / chargement)           | ✅     |
| Progression temps réel (vitesse moyennée, ETA fichier + total)       | ✅     |
| Annulation propre (suppression du fichier partiel)                   | ✅     |
| Pause / Reprise (effective au prochain fichier)                      | ✅     |
| Skip du fichier en cours                                             | ✅     |
| Protection à la fermeture si encodage actif                          | ✅     |
| Nettoyage intelligent de noms de fichiers (regex pré-compilées)      | ✅     |
| Ordre des tags du nom de sortie 100 % personnalisable                | ✅     |
| Tags désactivables individuellement                                  | ✅     |
| Casse configurable (titre, résolution, codec, source, provider)      | ✅     |
| Tag `team` insérable à n'importe quelle position                     | ✅     |
| Séparateur de tags configurable                                      | ✅     |
| Renommage assisté par fichier avec aperçu en temps réel              | ✅     |
| Extraction de sous-titres (SRT / ASS)                                | ✅     |
| Thème dark / light + palettes de couleurs                            | ✅     |
| Layout de navigation configurable (vertical / horizontal)            | ✅     |
| Préférences d'encodage persistantes (JSON)                           | ✅     |
| Config générale persistante (JSON)                                   | ✅     |
| Base de données SQLite (SeaORM) pour l'historique                    | ✅     |
| Dashboard de statistiques cumulées (encodage + extraction)           | ✅     |
| Historique détaillé des sessions et des fichiers encodés             | ✅     |
| Notifications Discord riches (embeds configurables)                  | ✅     |
| Champs Discord activables/désactivables par type de notification     | ✅     |
| Notes personnalisées dans les embeds Discord                         | ✅     |
| Bot Discord avec commandes à distance (Twilight gateway)             | ✅     |
| Reconnexion automatique du bot (backoff exponentiel, max 5 min)      | ✅     |

---

## Personnalisation du nom de sortie

Le panneau **Paramètres d'encodage → Tags** permet d'adapter entièrement le nom généré pour chaque fichier, sans toucher au code.

### Tags disponibles

| Id           | Description                                                   | Exemple                  |
| ------------ | ------------------------------------------------------------- | ------------------------ |
| `title`      | Titre nettoyé                                                 | `Jujutsu Kaisen`         |
| `se`         | Saison/Épisode (format choisi dans les réglages de renommage) | `S03E01`                 |
| `audio`      | Tag audio/langue détecté                                      | `VOSTFR`, `VF`, `MULTI`… |
| `resolution` | Résolution vidéo                                              | `1080P`                  |
| `provider`   | Provider détecté dans le nom source                           | `AMZN`, `NF`…            |
| `source`     | Source vidéo                                                  | `BluRay`, `WEB-DL`…      |
| `codec`      | Codec vidéo de sortie                                         | `H265`                   |
| `bitdepth`   | Profondeur de couleur                                         | `10bit`                  |
| `audioCodec` | Codec audio de sortie                                         | `AAC`, `EAC3`…           |
| `team`       | Tag libre, ignoré si vide                                     | `MaTeam`                 |

### Réordonner / désactiver

Chaque tag dispose de boutons ↑ / ↓ pour le déplacer et peut être désactivé individuellement. Le tag `team` est ignoré tant qu'aucun texte n'est saisi dans le champ correspondant.

**Exemple** avec l'ordre par défaut et la team `MaTeam` :

```
Jujutsu Kaisen S03E01 VOSTFR 1080P BluRay H265 10bit AAC MaTeam
```

---

## Configuration Discord

RenCodeX peut envoyer des notifications Discord via un bot et écouter des commandes à distance.

> **Bibliothèque** : le bot utilise **Twilight** (gateway WebSocket) avec rustls-native-roots — aucune dépendance à OpenSSL ni à vcpkg, fonctionnel out-of-the-box sur Windows MSVC.

### Connexion

**Via l'UI Settings → onglet Discord (prioritaire) :**

| Champ                | Rôle                                         |
| -------------------- | -------------------------------------------- |
| `Token du bot`       | Token Discord du bot                         |
| `Salon de logs`      | Channel pour les embeds de notification      |
| `Salon de commandes` | Channel écouté par le bot pour les commandes |

**Variables d'environnement (fallback si champs vides dans l'UI) :**

```powershell
$env:RENCODEX_DISCORD_TOKEN       = "Bot_token_ici"
$env:RENCODEX_DISCORD_LOG_CHANNEL = "ID_salon_logs"
$env:RENCODEX_DISCORD_CMD_CHANNEL = "ID_salon_commandes"
```

Le bot se reconnecte automatiquement en cas de déconnexion, avec un backoff exponentiel plafonné à 5 minutes. Il s'arrête après 5 échecs d'authentification consécutifs. La config est relue à chaque redémarrage du bot.

### Notifications disponibles

Chaque type de notification peut être activé/désactivé indépendamment. Les champs inclus dans chaque embed sont configurables individuellement, et une note personnalisée (avec variables dynamiques) peut être ajoutée en bas de chaque embed.

| Type        | Déclencheur                              | Champs disponibles                                   |
| ----------- | ---------------------------------------- | ---------------------------------------------------- |
| `start`     | Lancement de la file                     | Nb fichiers, taille totale, CRF, preset              |
| `file_done` | Fin de chaque fichier                    | Taille avant/après, gain, durée, CRF, preset         |
| `progress`  | Périodiquement (intervalle configurable) | Fichier, position, %, vitesse, temps restant, écoulé |
| `summary`   | Fin de session complète                  | Fichiers traités, gain total, durée, espace libéré   |
| `error`     | Échec d'encodage                         | Nom du fichier, message d'erreur                     |

### Commandes bot à distance

Le bot écoute le salon de commandes configuré et répond aux commandes suivantes :

| Commande  | Action                                                |
| --------- | ----------------------------------------------------- |
| `!status` | Affiche la progression en cours (barre, vitesse, ETA) |
| `!queue`  | Liste les fichiers en attente avec leur statut        |
| `!skip`   | Passe au fichier suivant (annule le fichier en cours) |
| `!pause`  | Demande une pause (effective au prochain fichier)     |
| `!resume` | Reprend après une pause                               |
| `!cancel` | Annule l'encodage et supprime le fichier partiel      |
| `!help`   | Affiche la liste des commandes                        |

---

## Extraction de sous-titres

L'onglet **Extraction** permet d'extraire les pistes de sous-titres d'un ou plusieurs fichiers vers des fichiers `.srt` ou `.ass`. Options disponibles :

- **Format** : SRT ou ASS
- **Nommage** : nom source du fichier ou nom nettoyé
- **Dossier de sortie** : même dossier que la source, Téléchargements, ou chemin personnalisé

---

## Presets d'encodage

Quatre presets intégrés (non supprimables) couvrent les cas d'usage courants :

| Preset        | CRF | Preset NVENC | Audio       | AQ         | Multipass |
| ------------- | --- | ------------ | ----------- | ---------- | --------- |
| **Rapide**    | 30  | p3           | Tout copier | Désactivée | Désactivé |
| **Équilibré** | 28  | p4           | Intelligent | —          | —         |
| **Qualité**   | 24  | p6           | Intelligent | Activée    | qres      |
| **Archive**   | 20  | p7           | Intelligent | Activée    | fullres   |

Des presets personnalisés peuvent être créés, nommés et sauvegardés depuis l'onglet _Presets_ du panneau d'encodage.

---

## Règles audio

Le mode audio **Intelligent** (par défaut) copie les codecs natifs déjà compatibles et réencode les formats lourds ou incompatibles :

| Codec source                        | Action           |
| ----------------------------------- | ---------------- |
| AAC, AC3, E-AC3, OPUS               | Copie directe    |
| DTS, TrueHD, FLAC, MP3, Vorbis, PCM | Réencodage → AAC |

Le mode **Tout copier** passe-through tous les streams sans réencodage. Le mode **Tout réencoder** normalise tous les streams en AAC.

---

## Stockage des données

### Fichiers de configuration — `%APPDATA%\RenCodeX\`

| Fichier               | Contenu                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------- |
| `config.json`         | Chemin FFmpeg, thème, layout, Discord, champs Discord, notes Discord, presets de dossier         |
| `encoding_prefs.json` | CRF, preset NVENC, format S/E, ordre des tags, team, règles audio, AQ, multipass, conteneur, extraction |

### Base de données SQLite — `%APPDATA%\RenCodeX\data.db`

La base est gérée par **SeaORM** et initialisée automatiquement au premier lancement. Elle remplace l'ancien `stats.json`.

| Table              | Contenu                                                            |
| ------------------ | ------------------------------------------------------------------ |
| `stats`            | Compteurs globaux (fichiers, sessions, Mo encodés, temps total…)   |
| `encode_sessions`  | Sessions d'encodage (date, nb fichiers, ratio, Mo économisés)      |
| `extract_sessions` | Sessions d'extraction (date, nb fichiers, nb pistes)               |
| `encoded_files`    | Détail de chaque fichier encodé (taille avant/après, ratio, durée) |
| `file_records`     | Records : fichier le plus lourd, meilleur ratio de compression     |

Les fichiers de config `config.json` et `encoding_prefs.json` sont volontairement séparés : sauvegarder les paramètres généraux n'écrase jamais les préférences d'encodage.

---

## Notes techniques

- **NVENC uniquement par défaut** : le codec utilisé est `hevc_nvenc`. Un GPU NVIDIA Kepler+ est requis. Pour encoder en CPU, remplacer `hevc_nvenc` par `libx265` dans `src-tauri/src/services/media.rs`.
- **Dolby Vision** : l'option `preserve_dv` tente de préserver le RPU DV. Le conteneur MKV est recommandé — MP4 ne supporte pas le DV RPU natif. Si la source ne contient pas de flux DV valide, le flag est inoffensif.
- **Pause/Reprise** : la pause prend effet au prochain fichier (pas en cours d'encodage). Une vraie suspension de processus via `CREATE_SUSPENDED` est envisageable mais non implémentée.
- **TLS** : toutes les connexions réseau (Discord, SMTP) utilisent **rustls** avec le provider **ring** — aucune dépendance à OpenSSL ou vcpkg.
- **Credentials** : aucun token Discord ni mot de passe SMTP n'est stocké dans les fichiers de config si les champs sont laissés vides. Les variables d'environnement peuvent être utilisées comme alternative.
- **Fermeture protégée** : si un encodage est en cours, RenCodeX affiche une boîte de dialogue de confirmation avant de se fermer et supprime proprement le fichier partiel.
