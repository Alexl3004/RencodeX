// ─── Types & Interfaces ───────────────────────────────────────────────────────

export interface StreamInfo {
  index: number;
  codec_type: string;
  codec_name: string;
  language: string;
  width?: number;
  height?: number;
}

export interface FileAnalysis {
  path: string;
  filename: string;
  size_mb: number;
  duration_secs: number;
  fps: number;
  streams: StreamInfo[];
  audio_langs: string[];
  sub_langs: string[];
}

export type AudioMode = "reencode" | "copy";
export type MultipassMode = "disabled" | "qres" | "fullres";
export type ContainerFormat = "mkv" | "mp4";

export type SubExtractFormat = "srt" | "ass";
export type SubExtractNaming = "source" | "cleaned";
export type SubExtractPathMode = "source" | "downloads" | "custom";

export interface EncodeJob {
  input_path: string;
  output_path: string;
  audio_langs: string[];
  sub_langs: string[];
  audio_overrides: Record<string, string>;
  sub_overrides: Record<string, string>;
  audio_codec_overrides: Record<string, string>;
  audio_bitrate_overrides: Record<string, string>;
  duration_secs: number;
  fps: number;
  crf: number;
  preset: string;
  audio_mode: AudioMode;
  audio_bitrate: number;
  spatial_aq: boolean;
  temporal_aq: boolean;
  aq_strength: number;
  multipass: MultipassMode;
  container: ContainerFormat;
}

export interface ProgressEvent {
  file_index: number;
  file_total: number;
  file_name: string;
  percent: number;
  speed: number;
  remaining_file: number;
  remaining_total: number;
}

export interface FileResult {
  path: string;
  name: string;
  status: "ok" | "error" | "cancelled";
  original_mb: number;
  encoded_mb: number;
  duration_secs: number;
  error_msg?: string;
}

export interface EncodeSummary {
  files: FileResult[];
  total_original_mb: number;
  total_encoded_mb: number;
  total_secs: number;
}

export interface CleanedName {
  title: string;
  year: string;
  season_episode: string;
  resolution: string;
  source: string;
  provider: string;
  audio_tags: string;
  suggested: string;
}

export interface AppFile {
  path: string;
  filename: string;
  size_mb: number;
  duration_secs: number;
  fps: number;
  audio_langs: string[];
  sub_langs: string[];
  streams: StreamInfo[];
  status:
    | "pending"
    | "analysing"
    | "ready"
    | "queued"
    | "encoding"
    | "done"
    | "error";
  output_name: string;
  output_ext: string;
  result?: FileResult;
  cleaned?: CleanedName;
  sub_extract_status: "none" | "extracting" | "done" | "error";
  sub_extract_error?: string;
}

export interface SubExtractProgress {
  file_index: number;
  file_total: number;
  file_name: string;
  percent: number;
}

// ─── Format Saison/Épisode ───────────────────────────────────────────────────

export type SeasonEpisodeFormat = "S01E01" | "S1E01" | "S1 E01" | "1x01";

// ─── Tag IDs & Options de nommage ────────────────────────────────────────────

export type TagId =
  | "title"
  | "se"
  | "audio"
  | "resolution"
  | "provider"
  | "source"
  | "codec"
  | "bitdepth"
  | "audioCodec"
  | "team"
  | "japver";

export type ResolutionCase  = "upper" | "lower";
export type TitleCaseMode   = "original" | "upper" | "lower" | "title";
export type CodecFormat     = "H265" | "H.265" | "HEVC";
export type SourceCase      = "original" | "upper" | "lower";
export type WebSourceFormat = "WEB-DL" | "WEBDL" | "Web-DL";
export type TagSeparator    = " " | "." | "_";
export type ProviderCase    = "upper" | "lower" | "hidden";

export interface NamingOptions {
  disabledTags?:    Set<TagId>;
  resolutionCase?:  ResolutionCase;
  titleCase?:       TitleCaseMode;
  codecFormat?:     CodecFormat;
  sourceCase?:      SourceCase;
  yearParentheses?: boolean;
  webSourceFormat?: WebSourceFormat;
  tagSeparator?:    TagSeparator;
  providerCase?:    ProviderCase;
  keepJapaneseVer?: boolean;
}
