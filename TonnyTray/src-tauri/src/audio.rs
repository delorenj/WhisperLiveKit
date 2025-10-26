use tauri::Emitter;
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use log::{debug, error, info, warn};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

use crate::events::AudioLevelEvent;

/// Audio manager for recording and playback
pub struct AudioManager {
    host: Host,
    input_device: Option<Device>,
    recording_stream: Arc<Mutex<Option<Stream>>>,
    app_handle: Arc<RwLock<Option<AppHandle>>>,
    last_event_time: Arc<RwLock<Instant>>,
    voice_threshold: f32,
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        info!("Audio host: {}", host.id().name());

        Ok(Self {
            host,
            input_device: None,
            recording_stream: Arc::new(Mutex::new(None)),
            app_handle: Arc::new(RwLock::new(None)),
            last_event_time: Arc::new(RwLock::new(Instant::now())),
            voice_threshold: 0.02, // Default voice activation threshold
        })
    }

    /// Set app handle for event emission
    pub async fn set_app_handle(&self, app_handle: AppHandle) {
        let mut handle = self.app_handle.write().await;
        *handle = Some(app_handle);
    }

    /// Set voice activation threshold
    pub fn set_voice_threshold(&mut self, threshold: f32) {
        self.voice_threshold = threshold;
    }

    /// Emit audio level event (throttled to ~10Hz)
    async fn emit_audio_level(&self, level: f32, peak: f32, is_speaking: bool) {
        // Check if we should emit (throttle to ~100ms = 10Hz)
        let should_emit = {
            let mut last_time = self.last_event_time.write().await;
            let now = Instant::now();
            let elapsed = now.duration_since(*last_time);

            if elapsed.as_millis() >= 100 {
                *last_time = now;
                true
            } else {
                false
            }
        };

        if should_emit {
            if let Some(ref app) = *self.app_handle.read().await {
                let event = AudioLevelEvent::new(level, peak, is_speaking);
                if let Err(e) = app.emit("audio_level", &event) {
                    error!("Failed to emit audio_level event: {}", e);
                }
            }
        }
    }

    /// List available input devices
    pub fn list_input_devices(&self) -> Result<Vec<String>> {
        let devices = self
            .host
            .input_devices()
            .context("Failed to enumerate input devices")?;

        let mut device_names = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_names.push(name);
            }
        }

        Ok(device_names)
    }

    /// List available output devices
    pub fn list_output_devices(&self) -> Result<Vec<String>> {
        let devices = self
            .host
            .output_devices()
            .context("Failed to enumerate output devices")?;

        let mut device_names = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_names.push(name);
            }
        }

        Ok(device_names)
    }

    /// Set input device by name
    pub fn set_input_device(&mut self, device_name: &str) -> Result<()> {
        let devices = self
            .host
            .input_devices()
            .context("Failed to enumerate input devices")?;

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_name {
                    info!("Selected input device: {}", name);
                    self.input_device = Some(device);
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!("Device not found: {}", device_name))
    }

    /// Get default input device
    pub fn get_default_input_device(&mut self) -> Result<()> {
        let device = self
            .host
            .default_input_device()
            .context("No default input device available")?;

        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Using default input device: {}", name);
        self.input_device = Some(device);
        Ok(())
    }

    /// Start recording audio
    pub fn start_recording<F>(&self, callback: F) -> Result<()>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let device = self
            .input_device
            .as_ref()
            .context("No input device selected")?;

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        debug!("Input config: {:?}", config);

        let callback = Arc::new(Mutex::new(callback));
        let app_handle = self.app_handle.clone();
        let threshold = self.voice_threshold;
        let last_event_time = self.last_event_time.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                self.build_input_stream_with_events::<f32>(device, &config.into(), callback, app_handle, threshold, last_event_time)?
            }
            cpal::SampleFormat::I16 => {
                self.build_input_stream_with_events::<i16>(device, &config.into(), callback, app_handle, threshold, last_event_time)?
            }
            cpal::SampleFormat::U16 => {
                self.build_input_stream_with_events::<u16>(device, &config.into(), callback, app_handle, threshold, last_event_time)?
            }
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play().context("Failed to start recording stream")?;

        let mut recording = self.recording_stream.lock().unwrap();
        *recording = Some(stream);

        info!("Recording started with audio level monitoring");
        Ok(())
    }

    /// Build input stream with event emission
    fn build_input_stream_with_events<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        callback: Arc<Mutex<dyn FnMut(&[f32]) + Send>>,
        app_handle: Arc<RwLock<Option<AppHandle>>>,
        threshold: f32,
        last_event_time: Arc<RwLock<Instant>>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Convert samples to f32
                    let samples: Vec<f32> = data.iter().map(|&s| cpal::Sample::from_sample(s)).collect();

                    // Calculate audio levels
                    let level = Self::calculate_audio_level(&samples);
                    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                    let is_speaking = level > threshold;

                    // Emit audio level event (with throttling)
                    let app_handle = app_handle.clone();
                    let last_event_time = last_event_time.clone();
                    tokio::spawn(async move {
                        // Check if we should emit (throttle to ~100ms = 10Hz)
                        let should_emit = {
                            let Ok(mut last_time) = last_event_time.try_write() else {
                                return;
                            };
                            let now = Instant::now();
                            let elapsed = now.duration_since(*last_time);

                            if elapsed.as_millis() >= 100 {
                                *last_time = now;
                                true
                            } else {
                                false
                            }
                        };

                        if should_emit {
                            if let Ok(handle_guard) = app_handle.try_read() {
                                if let Some(ref app) = *handle_guard {
                                    let event = AudioLevelEvent::new(level, peak, is_speaking);
                                    let _ = app.emit("audio_level", &event);
                                }
                            }
                        }
                    });

                    // Call the user callback with audio data
                    if let Ok(mut cb) = callback.lock() {
                        cb(&samples);
                    }
                },
                move |err| {
                    error!("Recording stream error: {}", err);
                },
                None,
            )
            .context("Failed to build input stream")?;

        Ok(stream)
    }

    /// Stop recording
    pub fn stop_recording(&self) -> Result<()> {
        let mut recording = self.recording_stream.lock().unwrap();
        *recording = None;
        info!("Recording stopped");
        Ok(())
    }

    /// Play audio from bytes (MP3, WAV, etc.)
    pub fn play_audio(&self, audio_data: Vec<u8>) -> Result<()> {
        // Create output stream fresh for playback (avoids thread safety issues)
        let (_stream, handle) = OutputStream::try_default()
            .context("Failed to create output stream")?;

        let cursor = Cursor::new(audio_data);
        let source = rodio::Decoder::new(cursor).context("Failed to decode audio")?;

        let sink = Sink::try_new(&handle).context("Failed to create audio sink")?;
        sink.append(source);

        // Block until playback finishes
        sink.sleep_until_end();

        info!("Playing audio");
        Ok(())
    }

    /// Get audio level from samples (RMS)
    pub fn calculate_audio_level(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();
        rms
    }

    /// Check if audio level exceeds threshold (voice activation)
    pub fn is_voice_detected(samples: &[f32], threshold: f32) -> bool {
        let level = Self::calculate_audio_level(samples);
        level > threshold
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("Failed to create default AudioManager: {}", e);
            panic!("Audio system initialization failed");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_creation() {
        let manager = AudioManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_list_devices() {
        let manager = AudioManager::new().unwrap();
        let devices = manager.list_input_devices();
        // Just ensure it doesn't panic
        assert!(devices.is_ok());
    }

    #[test]
    fn test_audio_level_calculation() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let level = AudioManager::calculate_audio_level(&samples);
        assert!(level > 0.0);
        assert!(level < 1.0);
    }

    #[test]
    fn test_voice_detection() {
        let silent = vec![0.001, 0.001, 0.001];
        let loud = vec![0.5, 0.5, 0.5];

        assert!(!AudioManager::is_voice_detected(&silent, 0.02));
        assert!(AudioManager::is_voice_detected(&loud, 0.02));
    }
}
