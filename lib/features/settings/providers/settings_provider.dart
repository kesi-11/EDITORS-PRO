import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../onboarding/providers/onboarding_provider.dart';

// ─── Settings State ───────────────────────────────────────────────

/// Immutable snapshot of all user-configurable settings.
class AppSettings {
  final String defaultResolution; // '720p', '1080p', '4K'
  final String defaultCodec; // 'H.264', 'H.265'
  final int autoSaveIntervalMinutes; // 1-10
  final bool autoSaveEnabled;
  final bool gpuAccelerationEnabled;
  final bool hardwareEncodingEnabled;
  final bool crashReportingEnabled;
  final bool analyticsEnabled;
  final String proxyQuality; // 'off', '360p', '480p', '720p'
  final int cacheSizeMb;
  final bool hardwareDecoding;

  const AppSettings({
    this.defaultResolution = '1080p',
    this.defaultCodec = 'H.264',
    this.autoSaveIntervalMinutes = 2,
    this.autoSaveEnabled = true,
    this.gpuAccelerationEnabled = true,
    this.hardwareEncodingEnabled = true,
    this.crashReportingEnabled = true,
    this.analyticsEnabled = false,
    this.proxyQuality = '480p',
    this.cacheSizeMb = 500,
    this.hardwareDecoding = true,
  });

  AppSettings copyWith({
    String? defaultResolution,
    String? defaultCodec,
    int? autoSaveIntervalMinutes,
    bool? autoSaveEnabled,
    bool? gpuAccelerationEnabled,
    bool? hardwareEncodingEnabled,
    bool? crashReportingEnabled,
    bool? analyticsEnabled,
    String? proxyQuality,
    int? cacheSizeMb,
    bool? hardwareDecoding,
  }) {
    return AppSettings(
      defaultResolution: defaultResolution ?? this.defaultResolution,
      defaultCodec: defaultCodec ?? this.defaultCodec,
      autoSaveIntervalMinutes:
          autoSaveIntervalMinutes ?? this.autoSaveIntervalMinutes,
      autoSaveEnabled: autoSaveEnabled ?? this.autoSaveEnabled,
      gpuAccelerationEnabled:
          gpuAccelerationEnabled ?? this.gpuAccelerationEnabled,
      hardwareEncodingEnabled:
          hardwareEncodingEnabled ?? this.hardwareEncodingEnabled,
      crashReportingEnabled:
          crashReportingEnabled ?? this.crashReportingEnabled,
      analyticsEnabled: analyticsEnabled ?? this.analyticsEnabled,
      proxyQuality: proxyQuality ?? this.proxyQuality,
      cacheSizeMb: cacheSizeMb ?? this.cacheSizeMb,
      hardwareDecoding: hardwareDecoding ?? this.hardwareDecoding,
    );
  }
}

// ─── Settings Notifier ────────────────────────────────────────────

/// Riverpod notifier that persists every settings change to
/// SharedPreferences immediately.
class SettingsNotifier extends StateNotifier<AppSettings> {
  final SharedPreferences _prefs;

  // Preference keys
  static const _keyResolution = 'settings_default_resolution';
  static const _keyCodec = 'settings_default_codec';
  static const _keyAutoSaveInterval = 'settings_auto_save_interval';
  static const _keyAutoSaveEnabled = 'settings_auto_save_enabled';
  static const _keyGpuAcceleration = 'settings_gpu_acceleration';
  static const _keyHardwareEncoding = 'settings_hardware_encoding';
  static const _keyCrashReporting = 'settings_crash_reporting';
  static const _keyAnalytics = 'settings_analytics';
  static const _keyProxyQuality = 'settings_proxy_quality';
  static const _keyCacheSizeMb = 'settings_cache_size_mb';
  static const _keyHardwareDecoding = 'settings_hardware_decoding';

  SettingsNotifier(this._prefs) : super(const AppSettings()) {
    _loadFromPrefs();
  }

  void _loadFromPrefs() {
    state = AppSettings(
      defaultResolution: _prefs.getString(_keyResolution) ?? '1080p',
      defaultCodec: _prefs.getString(_keyCodec) ?? 'H.264',
      autoSaveIntervalMinutes:
          _prefs.getInt(_keyAutoSaveInterval) ?? 2,
      autoSaveEnabled: _prefs.getBool(_keyAutoSaveEnabled) ?? true,
      gpuAccelerationEnabled:
          _prefs.getBool(_keyGpuAcceleration) ?? true,
      hardwareEncodingEnabled:
          _prefs.getBool(_keyHardwareEncoding) ?? true,
      crashReportingEnabled:
          _prefs.getBool(_keyCrashReporting) ?? true,
      analyticsEnabled: _prefs.getBool(_keyAnalytics) ?? false,
      proxyQuality: _prefs.getString(_keyProxyQuality) ?? '480p',
      cacheSizeMb: _prefs.getInt(_keyCacheSizeMb) ?? 500,
      hardwareDecoding:
          _prefs.getBool(_keyHardwareDecoding) ?? true,
    );
  }

  // ─── Mutators (each persists immediately) ────────────────────

  Future<void> setResolution(String value) async {
    await _prefs.setString(_keyResolution, value);
    state = state.copyWith(defaultResolution: value);
  }

  Future<void> setCodec(String value) async {
    await _prefs.setString(_keyCodec, value);
    state = state.copyWith(defaultCodec: value);
  }

  Future<void> setAutoSaveInterval(int minutes) async {
    await _prefs.setInt(_keyAutoSaveInterval, minutes);
    state = state.copyWith(autoSaveIntervalMinutes: minutes);
  }

  Future<void> setAutoSaveEnabled(bool enabled) async {
    await _prefs.setBool(_keyAutoSaveEnabled, enabled);
    state = state.copyWith(autoSaveEnabled: enabled);
  }

  Future<void> setGpuAcceleration(bool enabled) async {
    await _prefs.setBool(_keyGpuAcceleration, enabled);
    state = state.copyWith(gpuAccelerationEnabled: enabled);
  }

  Future<void> setHardwareEncoding(bool enabled) async {
    await _prefs.setBool(_keyHardwareEncoding, enabled);
    state = state.copyWith(hardwareEncodingEnabled: enabled);
  }

  Future<void> setCrashReporting(bool enabled) async {
    await _prefs.setBool(_keyCrashReporting, enabled);
    state = state.copyWith(crashReportingEnabled: enabled);
  }

  Future<void> setAnalytics(bool enabled) async {
    await _prefs.setBool(_keyAnalytics, enabled);
    state = state.copyWith(analyticsEnabled: enabled);
  }

  Future<void> setProxyQuality(String quality) async {
    await _prefs.setString(_keyProxyQuality, quality);
    state = state.copyWith(proxyQuality: quality);
  }

  Future<void> setCacheSizeMb(int mb) async {
    await _prefs.setInt(_keyCacheSizeMb, mb);
    state = state.copyWith(cacheSizeMb: mb);
  }

  Future<void> setHardwareDecoding(bool enabled) async {
    await _prefs.setBool(_keyHardwareDecoding, enabled);
    state = state.copyWith(hardwareDecoding: enabled);
  }
}

// ─── Provider ─────────────────────────────────────────────────────

/// Provider that exposes [SettingsNotifier] and the current [AppSettings].
final settingsProvider =
    StateNotifierProvider<SettingsNotifier, AppSettings>((ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  return SettingsNotifier(prefs);
});
