//! Analyse et encodage des fichiers média

use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use crate::naming::regex::{FFMPEG_FRAME, FFMPEG_SPEED, FFMPEG_OUT_TIME, normalize_hdr_tags};

use crate::models::{StreamInfo, EncodeJob, ProgressEvent, FileResult, EncodeSummary};
use std::sync::atomic::Ordering;
use crate::state::{
    CANCEL, PAUSE, ENCODING, CHILD_PID,
    PERCENT, SPEED, REMAINING, FILE_INDEX, FILE_TOTAL,
    store_f64, lock_meta,
};
use crate::utils::{filename_of, delete_partial_output, ffmpeg_path, ffprobe_path, normalize_lang, resolve_config};
use crate::services::notify::{
    discord_notify,
    discord_notify_start,
    discord_notify_file_done,
    discord_notify_error,
    discord_notify_progress,
};
use crate::commands::settings::load_config;

pub async fn analyze_file(path: String) -> Result<crate::models::FileAnalysis, String> {
    let ffprobe = ffprobe_path();
    let output = tokio::process::Command::new(&ffprobe)
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| format!("ffprobe error: {e}"))?;

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("JSON parse: {e}"))?;

    let format = &json["format"];
    let duration = format["duration"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let size_mb = std::fs::metadata(&path)
        .map(|m| m.len()).unwrap_or(0) as f64 / 1_048_576.0;

    let fps = json["streams"].as_array()
        .and_then(|streams| {
            streams.iter()
                .find(|s| s["codec_type"].as_str() == Some("video"))
                .and_then(|v| v["r_frame_rate"].as_str())
                .and_then(|r| {
                    let parts: Vec<f64> = r.split('/')
                        .filter_map(|x| x.parse().ok())
                        .collect();
                    if parts.len() == 2 && parts[1] > 0.0 {
                        Some(parts[0] / parts[1])
                    } else { None }
                })
        })
        .unwrap_or(25.0);

    let mut streams = Vec::new();
    let mut audio_langs = Vec::new();
    let mut sub_langs = Vec::new();

    for s in json["streams"].as_array().unwrap_or(&vec![]) {
        let codec_type = s["codec_type"].as_str().unwrap_or("").to_string();
        let codec_name = s["codec_name"].as_str().unwrap_or("").to_string();
        let index = s["index"].as_u64().unwrap_or(0) as u32;
        let language = normalize_lang(s["tags"]["language"].as_str().unwrap_or("und"));
        let width = s["width"].as_u64().map(|v| v as u32);
        let height = s["height"].as_u64().map(|v| v as u32);

        match codec_type.as_str() {
            "audio" => { if !audio_langs.contains(&language) { audio_langs.push(language.clone()); } }
            "subtitle" => { if !sub_langs.contains(&language) { sub_langs.push(language.clone()); } }
            _ => {}
        }
        streams.push(crate::models::StreamInfo { index, codec_type, codec_name, language, width, height });
    }

    let order = ["fre", "eng", "jpn", "kor", "ger", "spa", "ita", "por", "rus", "chi", "und"];
    let sort = |v: &mut Vec<String>| {
        v.sort_by_key(|l| order.iter().position(|o| *o == l).unwrap_or(999));
    };
    sort(&mut audio_langs);
    sort(&mut sub_langs);

    let filename = std::path::Path::new(&path).file_name()
        .and_then(|f| f.to_str()).unwrap_or("").to_string();

    // ── Extraction des métadonnées HDR depuis le premier stream vidéo ────
    let video_stream = json["streams"].as_array()
        .and_then(|streams| streams.iter().find(|s| s["codec_type"].as_str() == Some("video")));

    let color_primaries = video_stream
        .and_then(|v| v["color_primaries"].as_str())
        .unwrap_or("")
        .to_string();

    let color_transfer = video_stream
        .and_then(|v| v["color_transfer"].as_str())
        .unwrap_or("")
        .to_string();

    let color_space = video_stream
        .and_then(|v| v["color_space"].as_str())
        .unwrap_or("")
        .to_string();

    let hdr_from_name = normalize_hdr_tags(&filename);
    let hdr_format = if !hdr_from_name.is_empty() {
        hdr_from_name
    } else {
        match color_transfer.as_str() {
            "smpte2084"    => "HDR10".to_string(),  // PQ — HDR10 et HDR10+
            "arib-std-b67" => "HLG".to_string(),    // HLG broadcast
            _              => String::new(),         // SDR ou inconnu
        }
    };

    Ok(crate::models::FileAnalysis {
        path, filename, size_mb, duration_secs: duration, fps, streams, audio_langs, sub_langs,
        hdr_format,
        color_primaries,
        color_transfer,
        color_space,
    })
}

pub async fn start_encoding(
    app: AppHandle,
    jobs: Vec<EncodeJob>,
) -> Result<EncodeSummary, String> {
    // Initialisation de la session — ordre délibéré : ENCODING=true en dernier
    // pour qu'un snapshot concurrent voie un état déjà cohérent.
    {
        CANCEL.store(false, Ordering::Relaxed);
        // PAUSE n'a pas besoin d'être réinitialisé ici : kill_ffmpeg_process
        // et resume_ffmpeg_process le gèrent, mais on le remet par sécurité.
        PAUSE.store(false, Ordering::Relaxed);
        CHILD_PID.store(u32::MAX, Ordering::Relaxed);
        FILE_TOTAL.store(jobs.len(), Ordering::Relaxed);
        FILE_INDEX.store(0, Ordering::Relaxed);
        store_f64(&PERCENT, 0.0);
        store_f64(&SPEED, 0.0);
        store_f64(&REMAINING, 0.0);
        {
            let mut meta = lock_meta();
            meta.current_file = String::new();
            meta.current_out = None;
            meta.start_time = Some(Instant::now());
            meta.queue_names = jobs.iter().map(|j| filename_of(&j.input_path)).collect();
        }
        // Release en dernier : garantit la visibilité des stores précédents.
        ENCODING.store(true, Ordering::Release);
    }

    let ffmpeg = ffmpeg_path();
    let total = jobs.len();
    let start_all = Instant::now();
    let mut results: Vec<FileResult> = Vec::new();
    let durations: Vec<f64> = jobs.iter().map(|j| j.duration_secs).collect();

    // Charger la configuration avec resolve_config pour prendre en compte les variables d'environnement
    let cfg = resolve_config(load_config());
    
    // Notification de début d'encodage (si activée)
    if cfg.discord_enabled && cfg.discord_notify_start && !cfg.discord_bot_token.is_empty() && !cfg.discord_log_channel_id.is_empty() {
        let total_size_mb: f64 = jobs.iter()
            .filter_map(|j| std::fs::metadata(&j.input_path).ok().map(|m| m.len() as f64 / 1_048_576.0))
            .sum();
        let crf_value = jobs.first().map(|j| j.crf).unwrap_or(28);
        let preset_value = jobs.first().map(|j| j.preset.clone()).unwrap_or_else(|| "p5".to_string());
        let total_jobs = jobs.len();
        let token = cfg.discord_bot_token.clone();
        let channel = cfg.discord_log_channel_id.clone();
        
        let fields_start = crate::services::discord_fields::default_fields("start");
        let note_start = cfg.discord_custom_notes.get("start").cloned().unwrap_or_default();
        tokio::spawn(async move {
            discord_notify_start(
                &token,
                &channel,
                total_jobs,
                total_size_mb,
                crf_value,
                &preset_value,
                &fields_start,
                &note_start,
            ).await;
        });
    }

    for (idx, job) in jobs.iter().enumerate() {
        if CANCEL.load(Ordering::Relaxed) {
            results.push(FileResult {
                job_id: job.job_id.clone(),
                path: job.input_path.clone(),
                name: filename_of(&job.input_path),
                status: "cancelled".to_string(),
                original_mb: 0.0, encoded_mb: 0.0, duration_secs: 0.0, error_msg: None,
            });
            continue;
        }

        let file_start = Instant::now();
        let original_mb = std::fs::metadata(&job.input_path)
            .map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0);

        { lock_meta().current_out = Some(job.output_path.clone()); }

        // Build FFmpeg args
        let mut cmd_args: Vec<String> = vec![
            "-i".into(), job.input_path.clone(),
            
            "-map".into(), "0:v:0".into(),
        ];

        let audio_streams: Vec<&StreamInfo> = job.streams.iter().filter(|s| s.codec_type == "audio").collect();
        let mut audio_codec_args: Vec<String> = Vec::new();
        let mut audio_index = 0usize;
        for stream in &audio_streams {
            let raw_lang = normalize_lang(&stream.language);
            let eff_lang = job.audio_overrides.get(&stream.index.to_string())
                .map(|s| s.as_str()).unwrap_or(&raw_lang).to_string();

            let include = job.audio_langs.contains(&eff_lang)
                || (eff_lang == "und" && job.audio_langs.contains(&"und".to_string()));

            if include {
                cmd_args.push("-map".into());
                cmd_args.push(format!("0:{}", stream.index));
                if job.audio_mode == "copy" {
                    audio_codec_args.extend([
                        format!("-c:a:{}", audio_index), "copy".into(),
                        format!("-metadata:s:a:{}", audio_index), format!("language={}", eff_lang),
                    ]);
                } else {
                    audio_codec_args.extend([
                        format!("-c:a:{}", audio_index), "aac".into(),
                        format!("-b:a:{}", audio_index), format!("{}k", job.audio_bitrate),
                        format!("-metadata:s:a:{}", audio_index), format!("language={}", eff_lang),
                    ]);
                }
                audio_index += 1;
            }
        }
        if audio_index == 0 {
            cmd_args.extend(["-map".into(), "0:a".into()]);
            if job.audio_mode == "copy" {
                audio_codec_args.extend(["-c:a".into(), "copy".into()]);
            } else {
                audio_codec_args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{}k", job.audio_bitrate)]);
            }
        }

        let sub_streams: Vec<&StreamInfo> = job.streams.iter().filter(|s| s.codec_type == "subtitle").collect();
        let mut sub_mapped = false;
        let mut sub_index = 0usize;
        let mut sub_meta: Vec<String> = Vec::new();
        for stream in &sub_streams {
            let raw_lang = normalize_lang(&stream.language);
            let eff_lang = job.sub_overrides.get(&stream.index.to_string())
                .map(|s| s.as_str()).unwrap_or(&raw_lang).to_string();
            if job.sub_langs.contains(&eff_lang) {
                cmd_args.push("-map".into());
                cmd_args.push(format!("0:{}", stream.index));
                sub_meta.extend([
                    format!("-metadata:s:s:{}", sub_index), format!("language={}", eff_lang),
                ]);
                sub_index += 1;
                sub_mapped = true;
            }
        }
        if !sub_mapped { cmd_args.push("-sn".into()); }

        // ── Codec vidéo : copie ou réencodage NVENC ───────────────────────
        if job.video_mode == "copy" {
            // Remuxage pur : le flux vidéo est copié bit-à-bit, aucune dégradation.
            // Les paramètres CRF, preset, AQ et multipass sont ignorés.
            cmd_args.extend([
                "-c:v".into(), "copy".into(),
            ]);
        } else {
            // Paramètres d'encodage DYNAMIQUES (CRF et Preset depuis le job)
            cmd_args.extend([
                "-c:v".into(), "hevc_nvenc".into(),
                "-profile:v".into(), "main10".into(),
                "-pix_fmt".into(), "p010le".into(),
                "-preset".into(), job.preset.clone(),
                "-cq".into(), job.crf.to_string(),
            ]);

            // Préservation des métadonnées de couleur HDR
            // Si les champs sont remplis (source HDR), on les copie explicitement.
            // Si vides (source SDR), on ne touche pas — FFmpeg utilisera ses propres défauts.
            if !job.color_primaries.is_empty() {
                cmd_args.extend([
                    "-color_primaries".into(), job.color_primaries.clone(),
                    "-color_trc".into(),       job.color_transfer.clone(),
                    "-colorspace".into(),      job.color_space.clone(),
                ]);
            }
            // color_range : les sources BD/streaming HDR sont toujours "tv" (limited range).
            // On force "tv" si HDR, on laisse FFmpeg décider sinon.
            if !job.hdr_tag.is_empty() {
                cmd_args.extend(["-color_range".into(), "tv".into()]);
            }

            // Qualité NVENC avancée : AQ spatiale/temporelle + multipass
            cmd_args.extend([
                "-spatial-aq".into(), if job.spatial_aq { "1".into() } else { "0".into() },
                "-temporal-aq".into(), if job.temporal_aq { "1".into() } else { "0".into() },
            ]);
            if job.spatial_aq || job.temporal_aq {
                cmd_args.extend(["-aq-strength".into(), job.aq_strength.to_string()]);
            }
            let multipass_val = match job.multipass.as_str() {
                "qres" => "qres",
                "fullres" => "fullres",
                _ => "disabled",
            };
            cmd_args.extend(["-multipass".into(), multipass_val.into()]);
        }

        cmd_args.extend(audio_codec_args);
        if !sub_meta.is_empty() {
            cmd_args.extend(sub_meta);
            if job.container == "mp4" {
                // Le MP4 ne supporte pas la copie directe des sous-titres ASS/SRT
                cmd_args.extend(["-c:s".into(), "mov_text".into()]);
            } else {
                cmd_args.extend(["-c:s".into(), "copy".into()]);
            }
        }

        // Conteneur de sortie : MKV (matroska) ou MP4
        let container_fmt = if job.container == "mp4" { "mp4" } else { "matroska" };
        cmd_args.extend([
            "-map_metadata".into(), "0".into(),
            "-map_chapters".into(), "0".into(),
            "-f".into(), container_fmt.into(),
            "-progress".into(), "pipe:1".into(),
            "-nostats".into(),
            "-stats_period".into(), "1".into(),
            "-y".into(),
            job.output_path.clone(),
        ]);

        // Spawn ffmpeg
        let mut child = match Command::new(&ffmpeg)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                results.push(FileResult {
                    job_id: job.job_id.clone(),
                    path: job.input_path.clone(),
                    name: filename_of(&job.input_path),
                    status: "error".to_string(),
                    original_mb, encoded_mb: 0.0,
                    duration_secs: 0.0,
                    error_msg: Some(e.to_string()),
                });
                continue;
            }
        };

        // Lecture progression
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        // Drainer stderr en arrière-plan dans un buffer circulaire borné.
        //
        // Stratégie : on ne garde que les STDERR_MAX_LINES dernières lignes de ffmpeg,
        // tronquées à STDERR_MAX_LINE_BYTES chacune. Cela borne la mémoire à ~1 MB
        // dans le pire cas (500 lignes × 2 048 octets) quelle que soit la durée de
        // l'encodage, tout en conservant les lignes les plus récentes — les seules
        // utiles pour diagnostiquer une erreur ffmpeg.
        const STDERR_MAX_LINES: usize = 500;
        const STDERR_MAX_LINE_BYTES: usize = 2048;

        let stderr_buf = std::sync::Arc::new(
            std::sync::Mutex::new(std::collections::VecDeque::<String>::with_capacity(STDERR_MAX_LINES + 1))
        );
        let stderr_buf_clone = stderr_buf.clone();
        let stderr_task = {
            let stderr = child.stderr.take().unwrap();
            tokio::spawn(async move {
                let mut stderr_reader = BufReader::new(stderr);
                let mut err_line = String::new();
                while let Ok(n) = stderr_reader.read_line(&mut err_line).await {
                    if n == 0 { break; }
                    // Tronquer les lignes anormalement longues (bannières, dumps hex…)
                    let stored = if err_line.len() > STDERR_MAX_LINE_BYTES {
                        format!("{}…[tronqué]\n", &err_line[..STDERR_MAX_LINE_BYTES])
                    } else {
                        err_line.clone()
                    };
                    if let Ok(mut ring) = stderr_buf_clone.lock() {
                        if ring.len() >= STDERR_MAX_LINES {
                            ring.pop_front(); // éjecte la plus ancienne
                        }
                        ring.push_back(stored);
                    }
                    err_line.clear();
                }
            })
        };

        let re_frame = &*FFMPEG_FRAME;
        let re_speed = &*FFMPEG_SPEED;
        let re_out_time = &*FFMPEG_OUT_TIME;

        let mut current_frame: Option<f64> = None;
        let mut out_time_us: Option<f64> = None;
        let mut speed_val: Option<f64> = None;
        let mut speed_history: Vec<f64> = Vec::new();
        let mut last_emit = Instant::now();
        let mut last_discord_progress = Instant::now();
        let progress_interval = cfg.discord_progress_interval.max(10);
        let file_duration = job.duration_secs;
        let file_fps = job.fps.max(1.0);
        let total_frames = (file_duration * file_fps).max(1.0);
        let file_name = filename_of(&job.input_path);

        {
            // PID : Release pour que pause/kill voient le handle après le spawn.
            if let Some(pid) = child.id() {
                CHILD_PID.store(pid, Ordering::Release);
            }
            FILE_INDEX.store(idx, Ordering::Relaxed);
            // Métadonnées stables — lock bref, jamais depuis la boucle chaude.
            lock_meta().current_file = file_name.clone();
        }

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }

            // Chemin chaud : lecture atomique, aucun lock, aucune contention.
            if CANCEL.load(Ordering::Relaxed) { break; }

            let l = line.trim();

            if let Some(cap) = re_frame.captures(l) {
                if let Ok(f) = cap[1].parse::<f64>() {
                    current_frame = Some(f);
                }
            }
            if let Some(cap) = re_out_time.captures(l) {
                if let Ok(us) = cap[1].parse::<f64>() {
                    out_time_us = Some(us / 1_000_000.0);
                }
            }
            if let Some(cap) = re_speed.captures(l) {
                let s: f64 = cap[1].parse().unwrap_or(1.0);
                if s > 0.05 && s < 500.0 {
                    speed_history.push(s);
                    if speed_history.len() > 20 { speed_history.remove(0); }
                    speed_val = Some(speed_history.iter().sum::<f64>() / speed_history.len() as f64);
                }
            }

            let is_progress_line = l.starts_with("progress=");
            let should_emit = is_progress_line || last_emit.elapsed() >= Duration::from_millis(800);

            if should_emit {
                last_emit = Instant::now();

                let elapsed = file_start.elapsed().as_secs_f64();

                // Priorité : out_time_us > frame > vitesse*elapsed > linéaire
                let done = if let Some(t) = out_time_us.filter(|&t| t > 0.5) {
                    t.min(file_duration)
                } else if let Some(frame) = current_frame.filter(|&f| f > 0.0) {
                    (frame / file_fps).min(file_duration)
                } else if let Some(speed) = speed_val.filter(|&s| s > 0.0) {
                    (elapsed * speed).min(file_duration)
                } else {
                    // Dernier recours : progression linéaire conservative
                    (elapsed * 0.8).min(file_duration * 0.8)
                };

                let percent = if file_duration > 0.0 {
                    (done / file_duration * 100.0).clamp(0.0, 99.9)
                } else if total_frames > 0.0 {
                    (current_frame.unwrap_or(0.0) / total_frames * 100.0).clamp(0.0, 99.9)
                } else {
                    0.0
                };

                let remaining_next: f64 = durations[idx + 1..].iter().sum();

                let (remaining_file, remaining_total) = if let Some(avg_speed) = speed_val.filter(|&s| s > 0.01) {
                    // Estimation précise : basée sur la vitesse mesurée
                    let rf = ((file_duration - done) / avg_speed).max(0.0);
                    let rt = rf + remaining_next / avg_speed;
                    (rf, rt)
                } else if done > 0.5 && elapsed > 0.5 {
                    // Estimation par ratio temps-réel / avancement
                    let rate = elapsed / done;
                    let rf = (rate * (file_duration - done)).max(0.0);
                    let rt = rf + remaining_next * rate;
                    (rf, rt)
                } else {
                    // Pas assez de données
                    let rf = (file_duration - done).max(0.0);
                    let rt = rf + remaining_next;
                    (rf, rt)
                };

                let avg_speed = speed_val.unwrap_or(0.0);

                // Métriques de progression : stores atomiques indépendants,
                // pas de lock. La boucle chaude ci-dessus reste libre de lire
                // CANCEL sans contention même pendant ces écritures.
                store_f64(&PERCENT, percent);
                store_f64(&SPEED, avg_speed);
                store_f64(&REMAINING, remaining_total);

                let _ = app.emit("encode-progress", ProgressEvent {
                    file_index: idx,
                    file_total: total,
                    file_name: file_name.clone(),
                    percent,
                    speed: avg_speed,
                    remaining_file,
                    remaining_total,
                });

                if cfg.discord_enabled
                    && cfg.discord_notify_progress
                    && !cfg.discord_bot_token.is_empty()
                    && !cfg.discord_log_channel_id.is_empty()
                    && last_discord_progress.elapsed() >= Duration::from_secs(progress_interval)
                {
                    last_discord_progress = Instant::now();
                    let token = cfg.discord_bot_token.clone();
                    let channel = cfg.discord_log_channel_id.clone();
                    let fname = file_name.clone();
                    let elapsed_file = file_start.elapsed().as_secs_f64();
                    let pct = percent;
                    let spd = avg_speed;
                    let rem = remaining_total;
                    let fields_progress = crate::services::discord_fields::default_fields("progress");
                    let note_progress = cfg.discord_custom_notes.get("progress").cloned().unwrap_or_default();
                    tokio::spawn(async move {
                        discord_notify_progress(
                            &token, &channel, &fname,
                            idx, total, pct, spd, rem, elapsed_file,
                            &fields_progress,
                            &note_progress,
                        ).await;
                    });
                }
            }
        }

        let status = child.wait().await;
        // Attendre que la tâche stderr ait fini de lire avant d'utiliser le buffer.
        let _ = stderr_task.await;
        // Matérialise le buffer circulaire en une seule String pour les rapports d'erreur.
        // On ne joint que les dernières lignes (déjà bornées par STDERR_MAX_LINES).
        let ffmpeg_stderr: Option<String> = stderr_buf.lock().ok().and_then(|ring| {
            if ring.is_empty() { return None; }
            let joined: String = ring.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("");
            if joined.trim().is_empty() { None } else { Some(joined) }
        });
        let elapsed = file_start.elapsed().as_secs_f64();
        let cancelled = CANCEL.load(Ordering::Relaxed);
        let ok = status.map(|s| s.success()).unwrap_or(false) && !cancelled;

        if !ok && !cancelled {
            let deleted = delete_partial_output(&job.output_path);
            let out_name = filename_of(&job.output_path);
            if deleted {
                let _ = app.emit("partial-file-deleted", serde_json::json!({
                    "path": job.output_path,
                    "name": out_name,
                }));
            }
        }

        let encoded_mb = if ok {
            std::fs::metadata(&job.output_path)
                .map(|m| m.len() as f64 / 1_048_576.0)
                .unwrap_or(0.0)
        } else { 0.0 };

        if cfg.discord_enabled && !cfg.discord_bot_token.is_empty() && !cfg.discord_log_channel_id.is_empty() {
            let token = cfg.discord_bot_token.clone();
            let channel = cfg.discord_log_channel_id.clone();
            let file_name_clone = file_name.clone();
            
            if ok && cfg.discord_notify_file_done {
                let crf_val = job.crf;
                let preset_val = job.preset.clone();
                let fields_file_done = crate::services::discord_fields::default_fields("file_done");
                let note_file_done = cfg.discord_custom_notes.get("file_done").cloned().unwrap_or_default();
                tokio::spawn(async move {
                    discord_notify_file_done(
                        &token,
                        &channel,
                        &file_name_clone,
                        original_mb,
                        encoded_mb,
                        elapsed,
                        crf_val,
                        &preset_val,
                        &fields_file_done,
                        &note_file_done,
                    ).await;
                });
            } else if !ok && !cancelled && cfg.discord_notify_error {
                let fields_error = crate::services::discord_fields::default_fields("error");
                let note_error = cfg.discord_custom_notes.get("error").cloned().unwrap_or_default();
                // Extraire les dernières lignes utiles du stderr FFmpeg (max 800 chars
                // pour rester dans les limites d'un embed Discord).
                let err_msg = ffmpeg_stderr
                    .as_deref()
                    .unwrap_or("Échec de l'encodage")
                    .chars()
                    .rev()
                    .take(800)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();
                tokio::spawn(async move {
                    discord_notify_error(
                        &token,
                        &channel,
                        &file_name_clone,
                        &err_msg,
                        &fields_error,
                        &note_error,
                    ).await;
                });
            }
        }

        if ok {
            let gain = if original_mb > 0.0 {
                100.0 * (original_mb - encoded_mb) / original_mb
            } else { 0.0 };
            let _ = app.notification()
                .builder()
                .title("✅ Encodage terminé")
                .body(&format!(
                    "{} — {:.0}% de gain ({:.1} MB → {:.1} MB)",
                    file_name, gain, original_mb, encoded_mb,
                ))
                .show();
        } else if cancelled {
            let _ = app.notification()
                .builder()
                .title("⏹ Fichier annulé")
                .body(&format!("{} — fichier partiel supprimé", file_name))
                .show();
        } else {
            let _ = app.notification()
                .builder()
                .title("❌ Échec d'encodage")
                .body(&format!("{} — fichier partiel supprimé", file_name))
                .show();
        }

        results.push(FileResult {
            job_id: job.job_id.clone(), 
            path: job.input_path.clone(),
            name: file_name.clone(),
            status: if cancelled { "cancelled".to_string() }
                    else if ok { "ok".to_string() }
                    else { "error".to_string() },
            original_mb, encoded_mb, duration_secs: elapsed,
            error_msg: if ok || cancelled { None } else { ffmpeg_stderr.clone() },
        });

        let _ = app.emit("encode-file-done", serde_json::json!({
            "job_id":      job.job_id,                   
            "index":       idx,
            "path":        job.input_path,                
            "name":        filename_of(&job.input_path),
            "status":      if ok { "ok" } else { "error" },
            "original_mb": original_mb,
            "encoded_mb":  encoded_mb,
            "error_msg":   if ok || cancelled { serde_json::Value::Null }
                           else { serde_json::Value::String(
                               ffmpeg_stderr.clone().unwrap_or_default()
                           )},
        }));

        { lock_meta().current_out = None; }
    }

    // Résumé global
    let success_count = results.iter().filter(|r| r.status == "ok").count();
    let total_original: f64 = results.iter().map(|r| r.original_mb).sum();
    let total_encoded: f64 = results.iter().map(|r| r.encoded_mb).sum();
    let gain_pct = if total_original > 0.0 {
        100.0 * (total_original - total_encoded) / total_original
    } else { 0.0 };
    let elapsed_secs = start_all.elapsed().as_secs();

    let _ = app.notification()
        .builder()
        .title("📦 RenCodeX — File terminée")
        .body(&format!(
            "{}/{} fichiers réussis · Gain total {:.1}% · Durée {}m{}s",
            success_count, total, gain_pct,
            elapsed_secs / 60, elapsed_secs % 60,
        ))
        .show();

    let summary = EncodeSummary {
        files: results,
        total_original_mb: total_original,
        total_encoded_mb: total_encoded,
        total_secs: start_all.elapsed().as_secs_f64(),
    };

    // Teardown de session — même ordre que l'init : ENCODING=false en dernier.
    {
        store_f64(&PERCENT, 0.0);
        store_f64(&SPEED, 0.0);
        store_f64(&REMAINING, 0.0);
        {
            let mut meta = lock_meta();
            meta.current_file = String::new();
            meta.queue_names = Vec::new();
            meta.start_time = None;
        }
        ENCODING.store(false, Ordering::Release);
    }

    if cfg.discord_enabled
        && cfg.discord_notify_summary
        && !cfg.discord_bot_token.is_empty()
        && !cfg.discord_log_channel_id.is_empty()
    {
        let fields_summary = crate::services::discord_fields::default_fields("summary");
        let note_summary = cfg.discord_custom_notes.get("summary").map(|s| s.as_str()).unwrap_or("");
        discord_notify(&cfg.discord_bot_token, &cfg.discord_log_channel_id, &summary, &fields_summary, note_summary).await;
    }

    Ok(summary)
}