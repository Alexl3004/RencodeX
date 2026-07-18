import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { toasts } from "$lib/stores/toasts.svelte";
import { stats } from "./stats.svelte";
import type {
  ProgressEvent,
  EncodeSummary,
  SubExtractProgress,
  EncodeJob,
} from "./types";
import { formatDuration, formatMb, codecDefault } from "./naming";
import { prefs } from "./prefs.store.svelte";
import { filesStore } from "./files.store.svelte";

// ─── Génère un UUID v4 simple sans dépendance externe ────────────────────────

function uuid(): string {
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
  });
}

// ─── Store encodage ───────────────────────────────────────────────────────────

function createEncodingStore() {
  let encoding           = $state(false);
  let paused             = $state(false);
  let progress           = $state<ProgressEvent | null>(null);
  let summary            = $state<EncodeSummary | null>(null);
  let logs               = $state<{ msg: string; level: "info" | "warn" | "error" | "success" }[]>([]);

  // Extraction sous-titres
  let extractingSubs      = $state(false);
  let subExtractProgress  = $state<SubExtractProgress | null>(null);
  let cancelExtraction    = $state(false);

  // Ordre des chemins du batch en cours (pour résoudre file_index → path)
  let encodingFilePaths = $state<string[]>([]);
  let manuallyCancelled = false;

  // Map job_id → path — source de vérité pour la corrélation des events
  let jobIdToPath = new Map<string, string>();

  let _unlisten: UnlistenFn[] = [];

  // ─── Log ─────────────────────────────────────────────────────────────────

  function log(
    msg: string,
    level: "info" | "warn" | "error" | "success" = "info",
  ) {
    const ts = new Date().toLocaleTimeString("fr-FR");
    logs = [...logs, { msg: `[${ts}] ${msg}`, level }];
    if (logs.length > 500) logs = logs.slice(-400);
  }

  function clearLogs() { logs = []; }

  // ─── Écoute des événements Tauri ─────────────────────────────────────────

  async function listenEvents() {
    const u1 = await listen<ProgressEvent>("encode-progress", (e) => {
      progress = e.payload;
      const activePath = encodingFilePaths[e.payload.file_index];
      filesStore.files = filesStore.files.map((f) => {
        if (f.status === "queued" || f.status === "encoding") {
          return { ...f, status: f.path === activePath ? "encoding" : "queued" };
        }
        return f;
      });
    });

    const u2 = await listen<any>("encode-file-done", (e) => {
      const p = e.payload;

      // ── Corrélation exclusivement par job_id ─────────────────────────────
      // On retrouve le chemin exact depuis la map construite au moment du lancement.
      // Aucun risque de mismatch même si deux fichiers ont le même nom.
      const targetPath = jobIdToPath.get(p.job_id);
      if (!targetPath) {
        console.warn(`[encode-file-done] job_id inconnu : ${p.job_id}`);
        return;
      }

      filesStore.files = filesStore.files.map((f) => {
        if (f.path !== targetPath) return f;
        const updated = { ...f, status: p.status === "ok" ? "done" : "error" } as typeof f;
        if (p.status === "ok") updated.result = p;
        return updated;
      });

      if (p.status === "ok") {
        const gain =
          p.original_mb > 0
            ? (((p.original_mb - p.encoded_mb) / p.original_mb) * 100).toFixed(1)
            : null;
        const dur = formatDuration(p.duration_secs);
        log(
          `✓ ${p.name}` +
            (gain ? ` — gain ${gain}%` : "") +
            ` (${formatMb(p.encoded_mb)}` +
            (dur ? `, ${dur}` : "") +
            `)`,
          "success",
        );
      } else {
        log(
          `✗ ${p.name} — échec${p.error_msg ? ` : ${p.error_msg}` : ""}`,
          "error",
        );
      }
    });
    const u3 = await listen<boolean>("encode-paused", (e) => {
    paused = e.payload;
    if (e.payload) {
        log("Encodage mis en pause (Discord)", "info");
        toasts.warn("Encodage en pause");
    } else {
        log("Encodage repris (Discord)", "info");
        toasts.success("Encodage repris");
    }
});

    const u4 = await listen("encode-cancelled", () => {
        encoding = false;
        paused   = false;
        progress = null;
        log("Encodage annulé depuis Discord", "warn");
        toasts.warn("Encodage annulé via Discord");
    });

    _unlisten = [u1, u2, u3, u4];
  }

  // ─── Encodage ─────────────────────────────────────────────────────────────

  async function startEncoding(outputDir: string) {
    const files = filesStore.files;
    if (!files.length || encoding) return;
    encoding = true;
    summary  = null;
    manuallyCancelled = false;

    const readyFiles  = files.filter((f) => f.status === "ready");
    const targetFiles =
      filesStore.encodeSelectionMode && filesStore.selectedForEncoding.size > 0
        ? readyFiles.filter((f) => filesStore.selectedForEncoding.has(f.path))
        : readyFiles;

    if (targetFiles.length === 0) {
      encoding = false;
      return;
    }

    // ── Construire les jobs avec un job_id unique par fichier ─────────────
    jobIdToPath = new Map();

    const jobs: EncodeJob[] = targetFiles.map((f) => {
      const id = uuid();
      jobIdToPath.set(id, f.path);          // enregistrement dans la map

      const audioCodecOvr: Record<string, string> = {};
      f.streams
        .filter((s) => s.codec_type === "audio")
        .forEach((s) => {
          const target = filesStore.globalCodecOverride[s.codec_name.toLowerCase()];
          if (target && target !== codecDefault(s.codec_name)) {
            audioCodecOvr[s.index.toString()] = target;
          }
        });
      const outExt = prefs.container === "mp4" ? ".mp4" : ".mkv";
      const dir    = outputDir.replace(/[\\\/]+$/, "");
      return {
        job_id:                  id,         // ← transmis au backend
        input_path:              f.path,
        output_path:             `${dir}\\${f.output_name}${outExt}`,
        audio_langs:             [...(filesStore.fileSelAudio.get(f.path) ?? filesStore.selAudio)],
        sub_langs:               [...(filesStore.fileSelSubs.get(f.path)  ?? filesStore.selSubs)],
        audio_overrides:         filesStore.audioOverrides[f.path] ?? {},
        sub_overrides:           filesStore.subOverrides[f.path]   ?? {},
        streams: f.streams,
        audio_codec_overrides:   audioCodecOvr,
        audio_bitrate_overrides: {},
        duration_secs:           f.duration_secs,
        fps:                     f.fps ?? 25,
        crf:                     prefs.crf,
        preset:                  prefs.preset,
        video_mode:              prefs.videoMode,
        audio_mode:              prefs.audioMode,
        audio_bitrate:           prefs.audioBitrate,
        spatial_aq:              prefs.spatialAq,
        temporal_aq:             prefs.temporalAq,
        aq_strength:             prefs.aqStrength,
        multipass:               prefs.multipass,
        container:               prefs.container,
        color_primaries:         f.color_primaries ?? "",
        color_transfer:          f.color_transfer  ?? "",
        color_space:             f.color_space     ?? "",
        hdr_tag:                 f.hdr_format      ?? "",
      };
    });

    const totalSize = formatMb(targetFiles.reduce((s, f) => s + f.size_mb, 0));
    const selLabel  =
      filesStore.encodeSelectionMode && filesStore.selectedForEncoding.size > 0
        ? ` (sélection : ${filesStore.selectedForEncoding.size}/${readyFiles.length})`
        : "";
    log(
      `── Démarrage de l'encodage — ${jobs.length} fichier${jobs.length > 1 ? "s" : ""}${selLabel} (${totalSize}) ──`,
      "info",
    );

    filesStore.files = filesStore.files.map((f) =>
      targetFiles.some((t) => t.path === f.path) && f.status === "ready"
        ? { ...f, status: "queued" }
        : f,
    );
    jobs.forEach((j, i) => {
      const name = j.input_path.split(/[\\\/]/).pop() ?? j.input_path;
      log(`  [${i + 1}/${jobs.length}] En file : ${name}`, "info");
    });

    encodingFilePaths = targetFiles.map((f) => f.path);

    try {
      summary = await invoke<EncodeSummary>("start_encoding", { jobs });
      await stats.recordSummary(summary);

      const ok        = summary.files.filter((f) => f.status === "ok").length;
      const errors    = summary.files.filter((f) => f.status === "error").length;
      const cancelled = summary.files.filter((f) => f.status === "cancelled").length;
      const gain      =
        summary.total_original_mb > 0
          ? (
              ((summary.total_original_mb - summary.total_encoded_mb) /
                summary.total_original_mb) *
              100
            ).toFixed(1)
          : null;
      const savedMb = summary.total_original_mb - summary.total_encoded_mb;
      const dur     = formatDuration(summary.total_secs);

      log(
        `── Encodage terminé — ${ok}/${jobs.length} réussi${ok > 1 ? "s" : ""}` +
          (errors    > 0 ? `, ${errors} erreur${errors > 1 ? "s" : ""}`          : "") +
          (cancelled > 0 ? `, ${cancelled} annulé${cancelled > 1 ? "s" : ""}`   : "") +
          (gain ? ` — gain global ${gain}%` : "") +
          (dur  ? ` — durée ${dur}` : "") +
          ` ──`,
        errors > 0 && ok === 0 ? "error" : "success",
      );

      if (!manuallyCancelled) {
        if (errors > 0 && ok === 0)
          toasts.error(`Batch échoué — ${errors}/${jobs.length} erreurs`);
        else if (errors > 0 || cancelled > 0)
          toasts.warn(
            `${ok}/${jobs.length} réussis` +
              (gain ? ` · −${gain}%` : "") +
              (errors > 0 ? ` · ${errors} erreur${errors > 1 ? "s" : ""}` : ""),
          );
        else
          toasts.success(
            `${ok} fichier${ok > 1 ? "s" : ""} encodé${ok > 1 ? "s" : ""}` +
              (gain ? ` · −${gain}% (${formatMb(savedMb)})` : "") +
              (dur  ? ` · ${dur}` : ""),
          );
      }
    } catch (e) {
      log(`Erreur fatale encodage : ${e}`, "error");
      toasts.error(`Erreur fatale : ${e}`);
    } finally {
      encoding              = false;
      paused                = false;
      progress              = null;
      encodingFilePaths     = [];
      jobIdToPath           = new Map();   // nettoyage de la map
      filesStore.selectedForEncoding = new Set();
      filesStore.encodeSelectionMode = false;
    }
  }

  async function cancelEncoding() {
    if (paused) {
      await invoke("resume_encoding");
      paused = false;
    }
    manuallyCancelled = true;
    invoke("cancel_encoding");
    encoding = false;
    // Marquer les fichiers encore en attente / en cours comme annulés
    filesStore.files = filesStore.files.map((f) =>
      f.status === "queued" || f.status === "encoding"
        ? { ...f, status: "error" as const, result: { ...f.result, status: "cancelled" } as any }
        : f,
    );
    log("Encodage annulé par l'utilisateur", "warn");
    toasts.warn("Encodage annulé");
  }

  async function pauseEncoding() {
    if (!encoding || paused) return;
    const ok = await invoke<boolean>("pause_encoding");
    if (ok) {
      paused = true;
      log("Encodage mis en pause", "info");
      toasts.warn("Encodage en pause");
    } else {
      log("Impossible de mettre en pause (process introuvable ?)", "warn");
    }
  }

  async function resumeEncoding() {
    if (!encoding || !paused) return;
    const ok = await invoke<boolean>("resume_encoding");
    if (ok) {
      paused = false;
      log("Encodage repris", "info");
      toasts.success("Encodage repris");
    } else {
      log("Impossible de reprendre l'encodage", "warn");
    }
  }

  async function skipEncoding() {
    if (!encoding || paused) return;
    await invoke("skip_encoding");
    log("Fichier actuel ignoré, passage au suivant…", "info");
    toasts.warn("Fichier ignoré");
  }

  // ─── Extraction sous-titres ───────────────────────────────────────────────

  function cancelSubtitleExtraction() {
    if (!extractingSubs) return;
    cancelExtraction = true;
    log("Annulation de l'extraction demandée…", "warn");
  }

  async function startSubtitleExtraction(outputDir: string) {
    const files      = filesStore.files;
    const selSubs    = filesStore.selSubs;
    const readyFiles = files.filter((f) => f.status === "ready");

    if (readyFiles.length === 0 || selSubs.size === 0 || extractingSubs) return;

    const targetFiles =
      filesStore.selectedForExtraction.size > 0
        ? readyFiles.filter((f) => filesStore.selectedForExtraction.has(f.path))
        : readyFiles;
    if (targetFiles.length === 0) return;

    extractingSubs     = true;
    cancelExtraction   = false;
    subExtractProgress = null;

    filesStore.files = filesStore.files.map((f) => ({
      ...f,
      sub_extract_status: targetFiles.some((t) => t.path === f.path)
        ? "none"
        : f.sub_extract_status,
      sub_extract_error: targetFiles.some((t) => t.path === f.path)
        ? undefined
        : f.sub_extract_error,
    }));

    try {
      const total = targetFiles.length;
      for (let i = 0; i < total; i++) {
        if (cancelExtraction) {
          log("Extraction annulée par l'utilisateur", "warn");
          toasts.warn("Extraction annulée");
          break;
        }

        const file = targetFiles[i];
        subExtractProgress = { file_index: i, file_total: total, file_name: file.filename, percent: 0 };

        filesStore.files = filesStore.files.map((f) =>
          f.path === file.path ? { ...f, sub_extract_status: "extracting" } : f,
        );

        try {
          const tracks = await invoke<{ index: number; language: string; codec: string }[]>(
            "list_subtitle_tracks",
            { path: file.path },
          );

          let dir = "";
          if (prefs.subExtractPathMode === "source") {
            dir = file.path.split(/[\\\/]/).slice(0, -1).join("\\");
          } else if (prefs.subExtractPathMode === "downloads") {
            try {
              const pathApi = await import("@tauri-apps/api/path");
              dir = await pathApi.downloadDir();
            } catch {
              dir = outputDir;
            }
          } else {
            dir = prefs.subExtractCustomPath || outputDir;
          }
          dir = dir.replace(/[\\\/]+$/, "");

          let baseName = "";
          if (prefs.subExtractNaming === "source") {
            baseName =
              file.path.split(/[\\\/]/).pop()?.replace(/\.[^.]+$/, "") ?? "subtitles";
          } else {
            baseName =
              file.cleaned?.title
                ? `${file.cleaned.title} ${file.cleaned.season_episode} ${file.cleaned.source}`
                    .replace(/\s+/g, " ")
                    .trim()
                : file.path.split(/[\\\/]/).pop()?.replace(/\.[^.]+$/, "") ?? "subtitles";
          }

          const activeSubs = filesStore.fileSelSubs.get(file.path) ?? selSubs;
          const extracted: string[] = [];

          for (const track of tracks) {
            if (!activeSubs.has(track.language)) continue;
            const outputPath = `${dir}\\${baseName}.${track.language}.${prefs.subExtractFormat}`;
            await invoke("extract_subtitles", {
              sourcePath: file.path,
              trackIndex: track.index,
              outputPath,
            });
            extracted.push(track.language.toUpperCase());
          }

          filesStore.files = filesStore.files.map((f) =>
            f.path === file.path ? { ...f, sub_extract_status: "done" } : f,
          );

          if (extracted.length > 0) {
            toasts.success(
              `${extracted.join(", ")} · ${prefs.subExtractFormat.toUpperCase()} extrait${extracted.length > 1 ? "s" : ""}`,
              { title: file.output_name },
            );
          } else {
            toasts.warn("Aucune piste correspondant à la sélection", { title: file.output_name });
          }
          log(`Sous-titres extraits pour ${file.filename} [${extracted.join(", ")}]`, "success");
        } catch (e) {
          const errMsg = String(e);
          filesStore.files = filesStore.files.map((f) =>
            f.path === file.path
              ? { ...f, sub_extract_status: "error", sub_extract_error: errMsg }
              : f,
          );
          toasts.error(errMsg, { title: `Erreur — ${file.output_name}` });
          log(`Erreur extraction pour ${file.filename} : ${errMsg}`, "error");
        }
      }

      await stats.recordExtraction(filesStore.files, selSubs);
    } catch (e) {
      log(`Erreur globale lors de l'extraction : ${e}`, "error");
    } finally {
      extractingSubs     = false;
      cancelExtraction   = false;
      subExtractProgress = null;
    }
  }

  // ─── Reset ────────────────────────────────────────────────────────────────

  function clearSummary() {
    summary  = null;
    progress = null;
  }

  // ─── Exports ──────────────────────────────────────────────────────────────

  return {
    get encoding()           { return encoding; },
    get paused()             { return paused; },
    get progress()           { return progress; },
    get summary()            { return summary; },
    get logs()               { return logs; },
    get extractingSubs()     { return extractingSubs; },
    get subExtractProgress() { return subExtractProgress; },

    listenEvents,
    startEncoding,
    cancelEncoding,
    pauseEncoding,
    resumeEncoding,
    skipEncoding,
    startSubtitleExtraction,
    cancelSubtitleExtraction,
    clearSummary,
    clearLogs,
    log,
  };
}

export const encodingStore = createEncodingStore();