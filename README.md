# RenCodeX — Tauri 2 + Svelte 5 + Rust

Encodeur vidéo desktop **H265/HEVC NVENC** pour Windows, avec nettoyage intelligent de noms de fichiers entièrement personnalisable, notifications Discord riches, commandes bot à distance, rapports email et scan de dossiers récursif.

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

**Option B — Via l'interface :** modifier le chemin dans les Settings de l'application (onglet FFmpeg). Sauvegardé dans `%APPDATA%\RenCodeX\config.json`.

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
├── src-tauri/                        # Backend Rust
│   ├── src/
│   │   ├── main.rs                   # Point d'entrée Tauri + démarrage bot Discord
│   │   ├── models.rs                 # Structs partagées (AppConfig, EncodingPrefs, EncodeJob…)
│   │   ├── state.rs                  # État global de l'encodeur (Mutex partagé)
│   │   ├── regex.rs                  # Regex pré-compilées (once_cell) pour noms de fichiers
│   │   ├── utils.rs                  # Utilitaires : chemins, langues, title case, resolve_config
│   │   ├── filename.rs               # Logique de nettoyage et parsing de noms de fichiers
│   │   ├── media.rs                  # Analyse ffprobe + encodage FFmpeg async + progression
│   │   ├── notify.rs                 # Notifications Discord (bot + embeds) + bot Serenity + email SMTP
│   │   ├── discord_fields.rs         # Catalogue et valeurs par défaut des champs Discord
│   │   ├── commands.rs               # Commandes Tauri exposées au frontend (~20 commandes)
│   │   └── tests.rs                  # Tests unitaires (noms de fichiers, langues, title case)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                              # Frontend Svelte 5 (runes)
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +layout.ts
│   │   └── +page.svelte              # Shell principal + navigation
│   ├── components/
│   │   ├── TopBar.svelte             # Barre du haut : logo, statut, navigation
│   │   ├── DropZone.svelte           # Zone de dépôt de fichiers
│   │   ├── FileTable.svelte          # Table des fichiers en file d'attente
│   │   ├── FileLangModal.svelte      # Sélection des pistes audio/sous-titres par fichier
│   │   ├── FileModal.svelte          # Détail d'un fichier (streams, infos)
│   │   ├── FileRenameModal.svelte    # Renommage assisté avec aperçu du nom généré
│   │   ├── LangSelector.svelte       # Sélecteur de langues global
│   │   ├── LangPopover.svelte        # Popover de langue inline dans la table
│   │   ├── ProgressPanel.svelte      # Barre de progression + ETA
│   │   ├── ControlBar.svelte         # Boutons Encoder / Annuler
│   │   ├── EncodingSettings.svelte   # CRF, preset, ordre des tags, team, AQ, multipass…
│   │   ├── RenamingSettings.svelte   # Format S/E, casse, tags désactivables, codec affiché
│   │   ├── Dashboard.svelte          # Statistiques cumulées et historique des sessions
│   │   ├── LogConsole.svelte         # Console de logs en temps réel
│   │   ├── OutputDirPicker.svelte    # Sélecteur de dossier de sortie
│   │   ├── Settings.svelte           # Préférences générales (FFmpeg, Discord, email, layout)
│   │   └── ToastNotif.svelte         # Notifications toast
│   ├── lib/
│   │   ├── stores/
│   │   │   ├── encoder.svelte.ts     # Store principal (fichiers, encodage, prefs, types)
│   │   │   ├── stats.svelte.ts       # Statistiques cumulées persistantes
│   │   │   ├── theme.svelte.ts       # Thème dark/light + palettes de couleurs
│   │   │   └── toasts.svelte.ts      # File de notifications toast
│   │   └── utils.ts                  # Formatage durée / taille
│   └── app.css                       # Variables CSS globales (thème, radius, fonts)
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
| Détection et sélection des pistes audio & sous-titres | ✅ |
| Override de langue par piste (par fichier) | ✅ |
| Encodage H265 NVENC (CRF configurable, main10, p010le) | ✅ |
| AQ spatiale / temporelle + multipass NVENC | ✅ |
| Conteneur MKV ou MP4 | ✅ |
| Mode audio : réencodage AAC ou copie directe | ✅ |
| Progression temps-réel (vitesse moyennée, ETA fichier + total) | ✅ |
| Annulation propre (suppression du fichier partiel) | ✅ |
| Nettoyage intelligent de noms de fichiers (regex) | ✅ |
| Ordre des tags du nom de sortie 100% personnalisable | ✅ |
| Tags désactivables individuellement | ✅ |
| Casse configurable (titre, résolution, codec, source) | ✅ |
| Tag `team` insérable à n'importe quelle position | ✅ |
| Renommage assisté par fichier avec aperçu | ✅ |
| Extraction de sous-titres (SRT / ASS) | ✅ |
| Thème dark / light + palettes de couleurs | ✅ |
| Layout de navigation configurable (vertical / horizontal) | ✅ |
| Préférences d'encodage persistantes | ✅ |
| Config générale persistante | ✅ |
| Dashboard de statistiques cumulées | ✅ |
| Historique des sessions d'encodage et d'extraction | ✅ |
| Notifications Discord riches (embeds configurables) | ✅ |
| Champs Discord activables/désactivables par type de notification | ✅ |
| Notes personnalisées dans les embeds Discord | ✅ |
| Bot Discord avec commandes à distance | ✅ |
| Rapport email (SMTP) | ✅ |
| Notifications système Windows (fin de fichier, erreur) | ✅ |

---

## Personnalisation du nom de sortie

Le panneau **Paramètres d'encodage → Ordre des tags** permet d'adapter entièrement le nom généré pour chaque fichier, sans toucher au code.

### Tags disponibles

| Id | Description | Exemple |
|---|---|---|
| `title` | Titre nettoyé | `Jujutsu Kaisen` |
| `se` | Saison/Épisode (format choisi dans les réglages de renommage) | `S03E01` |
| `audio` | Tag audio/langue détecté | `VOSTFR`, `VF`, `MULTI`… |
| `resolution` | Résolution vidéo | `1080P` |
| `provider` | Provider détecté dans le nom source | `AMZN`, `NF`… |
| `source` | Source vidéo | `BluRay`, `WEB-DL`… |
| `codec` | Codec vidéo de sortie | `H265` |
| `bitdepth` | Profondeur de couleur | `10bit` |
| `audioCodec` | Codec audio de sortie | `AAC`, `EAC3`… |
| `team` | Tag libre, ignoré si vide | `MaTeam` |

### Réordonner / désactiver

Chaque tag dispose de boutons ↑ / ↓ pour le déplacer et peut être désactivé individuellement. Le tag `team` est ignoré tant qu'aucun texte n'est saisi dans le champ correspondant.

**Exemple** avec l'ordre par défaut et la team `MaTeam` :

```
Jujutsu Kaisen S03E01 VOSTFR 1080P BluRay H265 10bit AAC MaTeam
```

---

## Configuration Discord

RenCodeX peut envoyer des notifications Discord via un bot et écouter des commandes à distance.

### Connexion

**Variables d'environnement (prioritaires sur la config UI) :**

```powershell
$env:RENCODEX_DISCORD_TOKEN       = "Bot_token_ici"
$env:RENCODEX_DISCORD_LOG_CHANNEL = "ID_salon_logs"
$env:RENCODEX_DISCORD_CMD_CHANNEL = "ID_salon_commandes"
```

**Via l'UI Settings (onglet Discord) :**

| Champ | Rôle |
|---|---|
| `Token du bot` | Token Discord du bot |
| `Salon de logs` | Channel pour les embeds de notification |
| `Salon de commandes` | Channel écouté par le bot pour les commandes |

### Notifications disponibles

Chaque type de notification peut être activé/désactivé indépendamment. Les champs inclus dans chaque embed sont configurables individuellement, et une note personnalisée (avec variables dynamiques cliquables) peut être ajoutée en bas de chaque embed.

| Type | Déclencheur | Champs disponibles |
|---|---|---|
| `start` | Lancement de la file | Nb fichiers, taille totale, CRF, preset |
| `file_done` | Fin de chaque fichier | Taille avant/après, gain, durée, CRF, preset |
| `progress` | Périodiquement (intervalle configurable) | Fichier, position, %, vitesse, temps restant, écoulé |
| `summary` | Fin de session complète | Fichiers traités, gain total, durée, espace libéré, détail |
| `error` | Échec d'encodage | Nom du fichier, message d'erreur |

### Commandes bot à distance

Le bot écoute le salon de commandes configuré et répond aux commandes suivantes :

| Commande | Action |
|---|---|
| `!status` | Affiche la progression en cours (barre, vitesse, ETA) |
| `!queue` | Liste les fichiers en attente avec leur statut |
| `!skip` | Passe au fichier suivant (annule le fichier en cours) |
| `!pause` | Demande une pause (effective au prochain fichier) |
| `!resume` | Reprend après une pause |
| `!cancel` | Annule l'encodage et supprime le fichier partiel |
| `!help` | Affiche la liste des commandes |

---

## Configuration Email

Le rapport email est envoyé via SMTP à la fin d'une session. Les identifiants sont fournis depuis l'UI (non stockés dans la config principale).

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

## Extraction de sous-titres

L'onglet **Extraction** permet d'extraire les pistes de sous-titres d'un ou plusieurs fichiers vers des fichiers `.srt` ou `.ass`. Options disponibles :

- **Format** : SRT ou ASS
- **Nommage** : nom source du fichier ou nom nettoyé
- **Dossier de sortie** : même dossier que la source, Téléchargements, ou chemin personnalisé

---

## Stockage des préférences

Tous les fichiers de configuration sont stockés dans `%APPDATA%\RenCodeX\` :

| Fichier | Contenu |
|---|---|
| `config.json` | Chemin FFmpeg, thème, layout, Discord, email, champs Discord, notes Discord, presets de dossier |
| `encoding_prefs.json` | CRF, preset, format S/E, ordre des tags, team, mode audio, AQ, multipass, conteneur, extraction |
| `stats.json` | Statistiques cumulées d'encodage et d'extraction, historique des sessions |

Ces fichiers sont séparés volontairement : sauvegarder les Settings généraux n'écrase jamais les préférences d'encodage, et inversement.

---

## Notes

- **NVENC uniquement par défaut** : le codec utilisé est `hevc_nvenc`. Un GPU NVIDIA Kepler+ est requis. Pour encoder en CPU, remplacer `hevc_nvenc` par `libx265` dans `src-tauri/src/media.rs`.
- **Pause/Reprise** : la pause via `!pause` prend effet au prochain fichier (pas en cours d'encodage). Une vraie suspension de processus via `CREATE_SUSPENDED` est envisageable mais non implémentée.
- **Credentials** : aucun token Discord ni mot de passe SMTP n'est stocké dans les fichiers de config. Ils sont fournis via l'UI à chaque session ou via des variables d'environnement.