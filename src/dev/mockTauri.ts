/**
 * Dev-only harness: lets the React UI run in a normal browser (no Tauri shell)
 * with realistic fake backend data. Imported first by both entry points; the
 * whole body sits behind `import.meta.env.DEV`, so production builds drop it.
 */
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const w = window as unknown as Record<string, unknown>;

if (import.meta.env.DEV && !w.__TAURI_INTERNALS__) {
  const bind = (id: string, name: string, description: string, b: string) => ({
    id,
    name,
    description,
    default_binding: b,
    current_binding: b,
  });

  const settings: Record<string, unknown> = {
    settings_schema_version: 4,
    bindings: {
      transcribe: bind(
        "transcribe",
        "Transcribe",
        "Converts your speech into text.",
        "ctrl+shift+space",
      ),
      transcribe_with_post_process: bind(
        "transcribe_with_post_process",
        "Transcribe with Post-Processing",
        "Converts your speech into text and applies AI post-processing.",
        "option+shift+space",
      ),
      cancel: bind(
        "cancel",
        "Cancel",
        "Cancels the current recording.",
        "escape",
      ),
      paste_last: bind(
        "paste_last",
        "Paste Last Dictation",
        "Pastes your most recent dictation again.",
        "option+shift+v",
      ),
      command_mode: bind(
        "command_mode",
        "Command Mode",
        "Speak an instruction to rewrite the selected text.",
        "option+shift+c",
      ),
    },
    shortcut_activation: "hold_or_toggle",
    hold_threshold_ms: 300,
    audio_feedback: false,
    audio_feedback_volume: 0.5,
    sound_theme: "marimba",
    start_hidden: false,
    autostart_enabled: false,
    update_checks_enabled: true,
    show_whats_new_on_update: true,
    whats_new_last_seen_version: "0.11.0",
    selected_model: "parakeet-tdt-0.6b-v3",
    onboarding_completed: !location.search.includes("onboarding"),
    always_on_microphone: false,
    spoken_languages: ["en"],
    selected_microphone: null,
    selected_channel: null,
    clamshell_microphone: null,
    selected_output_device: null,
    translate_to_english: false,
    selected_language: "auto",
    overlay_position: "bottom",
    debug_mode: false,
    log_level: "info",
    custom_words: ["murmur", "Tauri"],
    model_unload_timeout: "min5",
    word_correction_threshold: 0.18,
    history_limit: 5,
    recording_retention_period: "preserve_limit",
    paste_method: "ctrl_v",
    clipboard_handling: "dont_modify",
    auto_submit: false,
    auto_submit_key: "enter",
    post_process_enabled: false,
    post_process_provider_id: "openai",
    post_process_providers: [
      {
        id: "openai",
        label: "OpenAI",
        base_url: "https://api.openai.com/v1",
        allow_base_url_edit: false,
        models_endpoint: "/models",
        supports_structured_output: true,
        api_format: "openai_chat",
      },
    ],
    post_process_api_keys: {},
    post_process_models: {},
    post_process_prompts: [
      {
        id: "default_improve_transcriptions",
        name: "Improve Transcriptions",
        prompt: "Clean up this transcript.\n${output}",
      },
    ],
    post_process_selected_prompt_id: null,
    mute_while_recording: false,
    append_trailing_space: false,
    app_language: "en",
    theme: "system",
    experimental_enabled: false,
    lazy_stream_close: false,
    airpods_mode: false,
    keyboard_implementation: "tauri",
    show_tray_icon: true,
    paste_delay_ms: 60,
    paste_delay_after_ms: 50,
    reliable_paste: false,
    typing_tool: "auto",
    external_script_path: null,
    filler_word_removal_enabled: true,
    custom_filler_words: null,
    chinese_script: "as_transcribed",
    transcribe_accelerator: "auto",
    transcribe_gpu_device: null,
    extra_recording_buffer_ms: 0,
    vad_enabled: true,
    vad_backend: "silero",
    long_form_chunking: false,
    language_detection: "when_needed",
    language_detection_window_secs: 5,
    overlay_style: "live",
    overlay_horizontal: "screen",
    overlay_offset_x: 0,
    overlay_size: "compact",
    overlay_accent: "default",
    overlay_opacity: 1,
    overlay_glass: "clear",
    overlay_glass_tint: "black",
    overlay_clear_tint: "clear",
    black_theme: false,
    prompt_chip_mode: "smart",
    prompt_chip_min_words: 12,
    voice_corrections_enabled: true,
    snippets: [{ trigger: "my email", text: "me@example.com" }],
    paste_last_shortcut_enabled: false,
    command_mode_enabled: false,
    app_styles: [{ bundle_id: "com.apple.mail", style: "formal" }],
  };

  const model = (id: string, name: string, size: number, down: boolean) => ({
    id,
    name,
    description: `${name} speech model.`,
    filename: `${id}.bin`,
    source: { Url: { url: `https://example.com/${id}.bin`, sha256: null } },
    size_mb: size,
    is_downloaded: down,
    is_downloading: false,
    partial_size: 0,
    accuracy_score: 0.8,
    speed_score: 0.7,
    supports_translation: false,
    is_recommended: id.startsWith("parakeet"),
    supported_languages: ["en", "de", "fr", "es"],
    supports_language_selection: true,
    is_custom: false,
    supports_streaming: false,
    supports_language_detection: true,
  });

  const models = [
    model("parakeet-tdt-0.6b-v3", "Parakeet v3", 640, true),
    model("whisper-small", "Whisper Small", 466, false),
    model("whisper-large-v3-turbo", "Whisper Large Turbo", 1600, false),
  ];

  const now = Math.floor(Date.now() / 1000);
  const history = [
    "Send the report to Anna before lunch.",
    "Remind me to call the dentist tomorrow morning.",
    "Let's move the meeting to Thursday at three.",
  ].map((text, i) => ({
    id: i + 1,
    file_name: `murmur-${i + 1}.wav`,
    timestamp: now - i * 3600,
    saved: i === 0,
    title: text.slice(0, 24),
    transcription_text: text,
    post_processed_text: null,
    post_process_prompt: null,
    post_process_requested: false,
    source_name: null,
    language: "en",
    language_source: "selected",
  }));

  const handlers: Record<string, () => unknown> = {
    get_app_settings: () => settings,
    get_default_settings: () => settings,
    get_available_models: () => models,
    get_current_model: () => settings.selected_model,
    get_retired_models: () => [],
    get_model_load_status: () => ({
      is_loaded: true,
      current_model: settings.selected_model,
    }),
    get_transcription_model_status: () => settings.selected_model,
    is_model_loading: () => false,
    is_language_detector_downloaded: () => true,
    get_history_entries: () => ({ entries: history, has_more: false }),
    get_usage_stats: () => ({
      total_words: 12840,
      words_this_week: 1520,
      dictations: 342,
      words_per_minute: 148,
      day_streak: 6,
    }),
    get_active_notices: () => [],
    get_available_microphones: () => [
      { index: "0", name: "MacBook Air Microphone", is_default: true },
    ],
    get_available_output_devices: () => [
      { index: "0", name: "MacBook Air Speakers", is_default: true },
    ],
    get_selected_microphone: () => "Default",
    get_selected_output_device: () => "Default",
    get_clamshell_microphone: () => "Default",
    get_microphone_mode: () => false,
    get_microphone_channels: () => 1,
    get_available_accelerators: () => ({ transcribe: ["auto"], gpu_devices: [] }),
    get_available_typing_tools: () => ["auto"],
    get_keyboard_implementation: () => "tauri",
    get_secure_input_status: () => false,
    get_app_dir_path: () => "/Users/me/Library/Application Support/murmur",
    get_log_dir_path: () => "/Users/me/Library/Logs/murmur",
    is_portable: () => false,
    is_laptop: () => true,
    is_update_checks_locked: () => false,
    is_recording: () => false,
    check_apple_intelligence_available: () => false,
    check_custom_sounds: () => ({ start: false, stop: false }),
    get_windows_microphone_permission_status: () => "allowed",
  };

  const settingKey = (cmd: string) => /^change_(.+)_setting$/.exec(cmd)?.[1];

  mockWindows("main");
  mockIPC(
    (cmd, args) => {
      const a = (args ?? {}) as Record<string, unknown>;
      if (cmd === "plugin:app|version") return "0.11.0";
      if (cmd === "plugin:app|name") return "murmur";
      if (cmd.startsWith("plugin:macos-permissions")) return true;
      if (cmd.startsWith("plugin:")) return null;
      if (handlers[cmd]) return handlers[cmd]();
      if (cmd === "get_model_info") {
        return models.find((m) => m.id === a.modelId) ?? null;
      }
      if (cmd === "change_appearance_setting") {
        const v = String(a.value);
        settings[String(a.key)] =
          v === "true"
            ? true
            : v === "false"
              ? false
              : v === "" || isNaN(+v)
                ? v
                : +v;
      } else if (cmd === "change_snippets_setting") {
        settings.snippets = a.snippets;
      } else if (cmd === "change_app_styles_setting") {
        settings.app_styles = a.styles;
      } else if (cmd === "update_custom_words") {
        settings.custom_words = a.words;
      } else {
        const key = settingKey(cmd);
        const vals = Object.values(a);
        if (key && key in settings && vals.length) settings[key] = vals[0];
      }
      return null;
    },
    { shouldMockEvents: true },
  );

  // plugin-os reads its platform from this global.
  w.__TAURI_OS_PLUGIN_INTERNALS__ = {
    platform: "macos",
    family: "unix",
    os_type: "macos",
    version: "15.0",
    arch: "aarch64",
    eol: "\n",
    exe_extension: "",
  };
  // murmur for QA from the console.
  w.__mockSettings = settings;
  w.__emit = (event: string, payload?: unknown) => emit(event, payload);
}
