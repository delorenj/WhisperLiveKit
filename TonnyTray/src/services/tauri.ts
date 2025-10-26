/**
 * Tauri IPC service layer
 * Provides type-safe wrappers for all Tauri commands and event listeners
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, Event as TauriEvent } from '@tauri-apps/api/event';
import type {
  AppSettings,
  AudioDevice,
  LogEntry,
  Transcription,
  Notification,
  ServerStatus,
  AudioLevel,
  Statistics,
  UserProfile,
  TauriEvents,
} from '@types';

/**
 * Generic invoke wrapper with error handling
 */
async function invokeCommand<T>(
  command: string,
  payload?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, payload);
  } catch (error) {
    console.error(`Tauri command '${command}' failed:`, error);
    throw error;
  }
}

/**
 * Recording control commands
 */
export const recordingCommands = {
  /**
   * Start recording with specified profile
   */
  async start(profileId: string): Promise<void> {
    return invokeCommand<void>('start_recording', { profileId });
  },

  /**
   * Stop current recording
   */
  async stop(save: boolean = true): Promise<void> {
    return invokeCommand<void>('stop_recording', { save });
  },

  /**
   * Pause current recording
   */
  async pause(): Promise<void> {
    return invokeCommand<void>('pause_recording');
  },

  /**
   * Resume paused recording
   */
  async resume(): Promise<void> {
    return invokeCommand<void>('resume_recording');
  },
};

/**
 * Settings management commands
 */
export const settingsCommands = {
  /**
   * Get current settings
   */
  async get(): Promise<AppSettings> {
    return invokeCommand<AppSettings>('get_settings');
  },

  /**
   * Update settings (partial or full)
   */
  async update(settings: Partial<AppSettings>): Promise<void> {
    return invokeCommand<void>('update_settings', { settings });
  },

  /**
   * Reset settings to defaults
   */
  async reset(): Promise<void> {
    return invokeCommand<void>('reset_settings');
  },

  /**
   * Export settings to file
   */
  async export(path: string): Promise<void> {
    return invokeCommand<void>('export_settings', { path });
  },

  /**
   * Import settings from file
   */
  async import(path: string): Promise<void> {
    return invokeCommand<void>('import_settings', { path });
  },
};

/**
 * Profile management commands
 */
export const profileCommands = {
  /**
   * Get all profiles
   */
  async getAll(): Promise<UserProfile[]> {
    return invokeCommand<UserProfile[]>('get_profiles');
  },

  /**
   * Get specific profile by ID
   */
  async get(profileId: string): Promise<UserProfile> {
    return invokeCommand<UserProfile>('get_profile', { profileId });
  },

  /**
   * Switch to different profile
   */
  async switch(profileId: string): Promise<void> {
    return invokeCommand<void>('switch_profile', { profileId });
  },

  /**
   * Create new profile
   */
  async create(profile: Omit<UserProfile, 'id'>): Promise<UserProfile> {
    return invokeCommand<UserProfile>('create_profile', { profile });
  },

  /**
   * Update existing profile
   */
  async update(profileId: string, profile: Partial<UserProfile>): Promise<void> {
    return invokeCommand<void>('update_profile', { profileId, profile });
  },

  /**
   * Delete profile
   */
  async delete(profileId: string): Promise<void> {
    return invokeCommand<void>('delete_profile', { profileId });
  },
};

/**
 * Audio device management commands
 */
export const audioCommands = {
  /**
   * Get available audio input devices
   */
  async getDevices(): Promise<AudioDevice[]> {
    return invokeCommand<AudioDevice[]>('get_audio_devices');
  },

  /**
   * Test audio device
   */
  async testDevice(deviceId: string): Promise<boolean> {
    return invokeCommand<boolean>('test_audio_device', { deviceId });
  },

  /**
   * Get current audio level
   */
  async getLevel(): Promise<AudioLevel> {
    return invokeCommand<AudioLevel>('get_audio_level');
  },
};

/**
 * Server management commands
 */
export const serverCommands = {
  /**
   * Start WhisperLiveKit server
   */
  async start(): Promise<void> {
    return invokeCommand<void>('start_server');
  },

  /**
   * Stop WhisperLiveKit server
   */
  async stop(): Promise<void> {
    return invokeCommand<void>('stop_server');
  },

  /**
   * Restart WhisperLiveKit server
   */
  async restart(): Promise<void> {
    return invokeCommand<void>('restart_server');
  },

  /**
   * Get server status
   */
  async getStatus(): Promise<ServerStatus> {
    return invokeCommand<ServerStatus>('get_server_status');
  },

  /**
   * Test server connection
   */
  async testConnection(): Promise<boolean> {
    return invokeCommand<boolean>('test_server_connection');
  },
};

/**
 * Integration commands
 */
export const integrationCommands = {
  /**
   * Test n8n webhook
   */
  async testWebhook(url: string, message: string = 'Test message'): Promise<boolean> {
    return invokeCommand<boolean>('test_n8n_webhook', { url, message });
  },

  /**
   * Get available ElevenLabs voices
   */
  async getVoices(): Promise<Array<{ id: string; name: string }>> {
    return invokeCommand<Array<{ id: string; name: string }>>('get_elevenlabs_voices');
  },

  /**
   * Test ElevenLabs TTS
   */
  async testTTS(text: string, voiceId: string): Promise<void> {
    return invokeCommand<void>('test_elevenlabs_tts', { text, voiceId });
  },

  /**
   * Send command to n8n
   */
  async sendCommand(command: string, profileId: string): Promise<string> {
    return invokeCommand<string>('send_command', { command, profileId });
  },
};

/**
 * Log management commands
 */
export const logCommands = {
  /**
   * Get logs with optional filtering
   */
  async get(
    level?: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<LogEntry[]> {
    return invokeCommand<LogEntry[]>('get_logs', { level, limit, offset });
  },

  /**
   * Clear all logs
   */
  async clear(): Promise<void> {
    return invokeCommand<void>('clear_logs');
  },

  /**
   * Export logs to file
   */
  async export(path: string): Promise<void> {
    return invokeCommand<void>('export_logs', { path });
  },
};

/**
 * History and statistics commands
 */
export const historyCommands = {
  /**
   * Get transcription history
   */
  async getTranscriptions(limit: number = 50): Promise<Transcription[]> {
    return invokeCommand<Transcription[]>('get_transcriptions', { limit });
  },

  /**
   * Get statistics
   */
  async getStatistics(): Promise<Statistics> {
    return invokeCommand<Statistics>('get_statistics');
  },

  /**
   * Clear history
   */
  async clearHistory(): Promise<void> {
    return invokeCommand<void>('clear_history');
  },
};

/**
 * System commands
 */
export const systemCommands = {
  /**
   * Open external URL
   */
  async openUrl(url: string): Promise<void> {
    return invokeCommand<void>('open_url', { url });
  },

  /**
   * Show notification
   */
  async showNotification(notification: Notification): Promise<void> {
    return invokeCommand<void>('show_notification', { notification });
  },

  /**
   * Check for updates
   */
  async checkForUpdates(): Promise<{ available: boolean; version?: string }> {
    return invokeCommand<{ available: boolean; version?: string }>(
      'check_for_updates'
    );
  },

  /**
   * Quit application
   */
  async quit(): Promise<void> {
    return invokeCommand<void>('quit_app');
  },
};

/**
 * Event listener types
 */
type EventCallback<T> = (event: TauriEvent<T>) => void;
type UnlistenFn = () => void;

/**
 * Event listeners
 */
export const eventListeners = {
  /**
   * Listen for new transcriptions
   */
  async onTranscription(
    callback: EventCallback<TauriEvents.TranscriptionEvent>
  ): Promise<UnlistenFn> {
    return listen('transcription', callback);
  },

  /**
   * Listen for status updates
   */
  async onStatusUpdate(
    callback: EventCallback<TauriEvents.StatusUpdateEvent>
  ): Promise<UnlistenFn> {
    return listen('status_update', callback);
  },

  /**
   * Listen for audio level updates
   */
  async onAudioLevel(
    callback: EventCallback<TauriEvents.AudioLevelEvent>
  ): Promise<UnlistenFn> {
    return listen('audio_level', callback);
  },

  /**
   * Listen for notifications
   */
  async onNotification(
    callback: EventCallback<TauriEvents.NotificationEvent>
  ): Promise<UnlistenFn> {
    return listen('notification', callback);
  },

  /**
   * Listen for errors
   */
  async onError(
    callback: EventCallback<TauriEvents.ErrorEvent>
  ): Promise<UnlistenFn> {
    return listen('error', callback);
  },
};

/**
 * Combined API export
 */
export const tauriApi = {
  recording: recordingCommands,
  settings: settingsCommands,
  profile: profileCommands,
  audio: audioCommands,
  server: serverCommands,
  integration: integrationCommands,
  logs: logCommands,
  history: historyCommands,
  system: systemCommands,
  events: eventListeners,
};

export default tauriApi;
