import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../data/models/project_model.dart';
import '../../../core/services/engine_service.dart';
import '../../../core/extensions/context_extensions.dart';
import 'engine_bridge_provider.dart';

// Re-export the bridge-generated DTOs so the rest of the app can
// reference them without knowing the bridge internals.
import 'package:editors_pro/src/rust/api/bridge_api.dart'
    show BridgeProjectSettings, ClipInfo, MediaAssetInfo, ProjectInfo;

/// Editor state
class EditorState {
  final bool isPlaying;
  final int currentTimeMs;
  final int durationMs;
  final double zoomLevel;
  final bool isImporting;
  final bool isExporting;
  final double exportProgress;
  final String? selectedClipId;
  final String? selectedTrackId;
  final LeftPanelTab leftPanelTab;
  final bool showInspector;
  final double playbackSpeed;
  final String? lastError;
  /// Whether audio playback is active (synchronized with video)
  final bool isAudioPlaying;
  /// Master volume level for audio playback (0.0 to 1.0)
  final double masterVolume;
  /// Whether the editor is currently fetching frames in the playback loop
  final bool isDecodingFrame;

  const EditorState({
    this.isPlaying = false,
    this.currentTimeMs = 0,
    this.durationMs = 0,
    this.zoomLevel = 1.0,
    this.isImporting = false,
    this.isExporting = false,
    this.exportProgress = 0,
    this.selectedClipId,
    this.selectedTrackId,
    this.leftPanelTab = LeftPanelTab.media,
    this.showInspector = false,
    this.playbackSpeed = 1.0,
    this.lastError,
    this.isAudioPlaying = false,
    this.masterVolume = 1.0,
    this.isDecodingFrame = false,
  });

  EditorState copyWith({
    bool? isPlaying,
    int? currentTimeMs,
    int? durationMs,
    double? zoomLevel,
    bool? isImporting,
    bool? isExporting,
    double? exportProgress,
    String? selectedClipId,
    String? selectedTrackId,
    LeftPanelTab? leftPanelTab,
    bool? showInspector,
    double? playbackSpeed,
    String? lastError,
    bool clearError = false,
    bool? isAudioPlaying,
    double? masterVolume,
    bool? isDecodingFrame,
  }) {
    return EditorState(
      isPlaying: isPlaying ?? this.isPlaying,
      currentTimeMs: currentTimeMs ?? this.currentTimeMs,
      durationMs: durationMs ?? this.durationMs,
      zoomLevel: zoomLevel ?? this.zoomLevel,
      isImporting: isImporting ?? this.isImporting,
      isExporting: isExporting ?? this.isExporting,
      exportProgress: exportProgress ?? this.exportProgress,
      selectedClipId: selectedClipId,
      selectedTrackId: selectedTrackId,
      leftPanelTab: leftPanelTab ?? this.leftPanelTab,
      showInspector: showInspector ?? this.showInspector,
      playbackSpeed: playbackSpeed ?? this.playbackSpeed,
      lastError: clearError ? null : (lastError ?? this.lastError),
      isAudioPlaying: isAudioPlaying ?? this.isAudioPlaying,
      masterVolume: masterVolume ?? this.masterVolume,
      isDecodingFrame: isDecodingFrame ?? this.isDecodingFrame,
    );
  }
}

enum LeftPanelTab { media, effects, text, audio }

/// Editor state notifier — mediates between the UI and the Rust engine.
///
/// Phase 4 improvements:
/// - Real-time preview playback with continuous frame decode loop
/// - Clip drag-and-move via engine bridge
/// - Project save/load integration
class EditorNotifier extends StateNotifier<EditorState> {
  Timer? _playbackTimer;
  DateTime? _lastFrameTime;
  final Ref _ref;

  EditorNotifier(this._ref) : super(const EditorState());

  @override
  void dispose() {
    _playbackTimer?.cancel();
    super.dispose();
  }

  /// Whether the engine is available for use.
  bool get _engineReady => EngineService.instance.isInitialized;

  /// Initialize the editor state from the engine.
  Future<void> initialize() async {
    if (!_engineReady) return;
    await _syncDurationFromEngine();
    state = const EditorState();
  }

  // ─── Playback ────────────────────────────────────────────────────

  /// Play/pause toggle
  void togglePlayback() {
    if (state.isPlaying) {
      _stopPlayback();
    } else {
      state = state.copyWith(isPlaying: true);
      _startPlayback();
    }
  }

  /// Stop playback and cancel timer
  void _stopPlayback() {
    _playbackTimer?.cancel();
    _playbackTimer = null;
    _lastFrameTime = null;
    state = state.copyWith(isPlaying: false, isAudioPlaying: false);
  }

  /// Start real-time playback from current position.
  ///
  /// Uses a high-frequency Timer that advances the playhead based on
  /// real elapsed time (not fixed intervals), providing smooth playback
  /// that accounts for frame decode latency.
  void _startPlayback() {
    _playbackTimer?.cancel();
    _lastFrameTime = DateTime.now();

    // Use ~60Hz tick for smooth playback
    const tickMs = 16;
    _playbackTimer = Timer.periodic(const Duration(milliseconds: tickMs), (timer) {
      if (!state.isPlaying) {
        timer.cancel();
        _playbackTimer = null;
        return;
      }

      final now = DateTime.now();
      final elapsed = _lastFrameTime != null
          ? now.difference(_lastFrameTime!).inMilliseconds
          : tickMs;
      _lastFrameTime = now;

      // Apply playback speed multiplier
      final adjustedElapsed = (elapsed * state.playbackSpeed).round();

      final newTime = state.currentTimeMs + adjustedElapsed;
      if (newTime >= state.durationMs) {
        // Loop back to start when reaching the end
        seekTo(0);
      } else {
        seekTo(newTime);
      }
    });
  }

  /// Seek to a specific time position
  void seekTo(int timeMs) {
    state = state.copyWith(
      currentTimeMs: timeMs.clamp(0, state.durationMs > 0 ? state.durationMs : 300000),
    );
  }

  /// Set the timeline duration
  void setDuration(int durationMs) {
    state = state.copyWith(durationMs: durationMs);
  }

  /// Set playback speed (0.25x to 4.0x)
  void setPlaybackSpeed(double speed) {
    state = state.copyWith(playbackSpeed: speed.clamp(0.25, 4.0));
  }

  // ─── Zoom ────────────────────────────────────────────────────────

  /// Set zoom level
  void setZoom(double zoom) {
    state = state.copyWith(zoomLevel: zoom.clamp(0.1, 10.0));
  }

  /// Zoom in
  void zoomIn() {
    setZoom(state.zoomLevel * 1.2);
  }

  /// Zoom out
  void zoomOut() {
    setZoom(state.zoomLevel / 1.2);
  }

  // ─── Selection ───────────────────────────────────────────────────

  /// Select a clip
  void selectClip(String? clipId) {
    state = state.copyWith(
      selectedClipId: clipId,
      showInspector: clipId != null,
    );
  }

  /// Select a track
  void selectTrack(String? trackId) {
    state = state.copyWith(
      selectedTrackId: trackId,
      showInspector: trackId != null,
    );
  }

  // ─── State flags ─────────────────────────────────────────────────

  /// Set importing state
  void setImporting(bool importing) {
    state = state.copyWith(isImporting: importing);
  }

  /// Set exporting state
  void setExporting(bool exporting, {double progress = 0}) {
    state = state.copyWith(isExporting: exporting, exportProgress: progress);
  }

  /// Set the left panel tab
  void setLeftPanelTab(LeftPanelTab tab) {
    state = state.copyWith(leftPanelTab: tab);
  }

  /// Toggle inspector visibility
  void toggleInspector() {
    state = state.copyWith(showInspector: !state.showInspector);
  }

  /// Set master volume for audio playback
  void setMasterVolume(double volume) {
    state = state.copyWith(masterVolume: volume.clamp(0.0, 1.0));
  }

  // ─── Bridge-wired operations ─────────────────────────────────────

  /// Undo last action via the Rust engine.
  Future<void> undo() async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.undo();
      await _syncDurationFromEngine();
      await _ref.read(projectProvider.notifier).syncFromEngine();
    } catch (e) {
      developer.log('undo failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Undo failed: $e');
    }
  }

  /// Redo last undone action via the Rust engine.
  Future<void> redo() async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.redo();
      await _syncDurationFromEngine();
      await _ref.read(projectProvider.notifier).syncFromEngine();
    } catch (e) {
      developer.log('redo failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Redo failed: $e');
    }
  }

  /// Split the currently selected clip at the playhead via the Rust engine.
  Future<void> splitAtPlayhead() async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.splitClip(clipId: clipId, timeMs: BigInt.from(state.currentTimeMs));
      await _syncDurationFromEngine();
      _refreshEngineState();
      await _ref.read(projectProvider.notifier).syncFromEngine();
    } catch (e) {
      developer.log('splitAtPlayhead failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Split failed: $e');
    }
  }

  /// Delete the currently selected clip via the Rust engine.
  Future<void> deleteSelected() async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.removeClip(clipId: clipId);
      state = state.copyWith(selectedClipId: null, showInspector: false);
      await _syncDurationFromEngine();
      _refreshEngineState();
      await _ref.read(projectProvider.notifier).syncFromEngine();
    } catch (e) {
      developer.log('deleteSelected failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Delete failed: $e');
    }
  }

  /// Import a media file via the Rust engine.
  Future<MediaAssetInfo?> importMedia(String filePath) async {
    if (!_engineReady) return null;
    try {
      final api = EngineService.instance.api;
      final assetInfo = await api.importMedia(filePath: filePath);
      await _syncDurationFromEngine();
      _refreshEngineState();
      return assetInfo;
    } catch (e) {
      developer.log('importMedia failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Import failed: $e');
      return null;
    }
  }

  /// Add a clip to a track via the Rust engine.
  Future<ClipInfo?> addClipToTrack({
    required String trackId,
    required String assetId,
    required int startMs,
    int durationMs = 0,
  }) async {
    if (!_engineReady) return null;
    try {
      final api = EngineService.instance.api;
      final clipInfo = await api.addClip(
        trackId: trackId,
        assetId: assetId,
        startMs: BigInt.from(startMs),
        durationMs: BigInt.from(durationMs),
      );
      await _syncDurationFromEngine();
      _refreshEngineState();
      return clipInfo;
    } catch (e) {
      developer.log('addClipToTrack failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Add clip failed: $e');
      return null;
    }
  }

  /// Move a clip to a new position on the timeline.
  ///
  /// This is the engine-backed implementation for clip drag-and-drop.
  Future<void> moveClip({
    required String clipId,
    required int newStartMs,
    String? newTrackId,
  }) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.moveClip(
        clipId: clipId,
        newStartMs: BigInt.from(newStartMs),
        newTrackId: newTrackId,
      );
      await _syncDurationFromEngine();
      _refreshEngineState();
      await _ref.read(projectProvider.notifier).syncFromEngine();
    } catch (e) {
      developer.log('moveClip failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Move clip failed: $e');
    }
  }

  /// Trim a clip by adjusting its start/end trim points.
  Future<void> trimClip({
    required String clipId,
    required int trimStartMs,
    required int trimEndMs,
  }) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.trimClip(
        clipId: clipId,
        trimStartMs: BigInt.from(trimStartMs),
        trimEndMs: BigInt.from(trimEndMs),
      );
      await _syncDurationFromEngine();
      _refreshEngineState();
    } catch (e) {
      developer.log('trimClip failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Trim clip failed: $e');
    }
  }

  /// Create a new project via the Rust engine.
  Future<ProjectInfo?> createProject(String name, {int? width, int? height, double? fps}) async {
    if (!_engineReady) return null;
    try {
      final api = EngineService.instance.api;

      BridgeProjectSettings? settings;
      if (width != null || height != null || fps != null) {
        settings = BridgeProjectSettings(
          width: width ?? 1920,
          height: height ?? 1080,
          fps: fps ?? 30.0,
        );
      }

      final projectInfo = await api.createProject(name: name, settings: settings);
      await _syncDurationFromEngine();
      _refreshEngineState();
      return projectInfo;
    } catch (e) {
      developer.log('createProject failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Create project failed: $e');
      return null;
    }
  }

  /// Save the current project via the Rust engine.
  Future<void> saveProject({required String filePath}) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.saveProject(filePath: filePath);
    } catch (e) {
      developer.log('saveProject failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Save failed: $e');
    }
  }

  /// Load a project from a .epp file via the Rust engine.
  Future<void> loadProject({required String filePath}) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.loadProject(filePath: filePath);
      await _syncDurationFromEngine();
      _refreshEngineState();
    } catch (e) {
      developer.log('loadProject failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Load failed: $e');
    }
  }

  // ─── Audio Bridge Operations ──────────────────────────────────────

  /// Set the volume level for a track via the Rust engine.
  Future<void> setTrackVolume(String trackId, double volume) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.setTrackVolume(trackId: trackId, volume: volume);
      _refreshEngineState();
    } catch (e) {
      developer.log('setTrackVolume failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Set volume failed: $e');
    }
  }

  /// Toggle track visibility (mute/unmute) via the Rust engine.
  Future<void> toggleTrackVisibility(String trackId) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.toggleTrackVisibility(trackId: trackId);
      _refreshEngineState();
    } catch (e) {
      developer.log('toggleTrackVisibility failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Toggle mute failed: $e');
    }
  }

  /// Get waveform peak data for an audio asset.
  Future<List<double>> getWaveform(String assetId, {int numBins = 200}) async {
    if (!_engineReady) return [];
    try {
      final api = EngineService.instance.api;
      final peaks = await api.getWaveform(assetId: assetId, numBins: numBins);
      return peaks.map((e) => e.toDouble()).toList();
    } catch (e) {
      developer.log('getWaveform failed: $e', name: 'EditorNotifier');
      return [];
    }
  }

  /// Configure audio ducking for a track.
  Future<void> setDucking(String trackId, {required bool enabled, double duckLevel = 0.3}) async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.setDucking(trackId: trackId, enabled: enabled, duckLevel: duckLevel);
    } catch (e) {
      developer.log('setDucking failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Set ducking failed: $e');
    }
  }

  /// Get the full timeline state from the engine.
  Future<dynamic> getTimelineState() async {
    if (!_engineReady) return null;
    try {
      final api = EngineService.instance.api;
      return await api.getTimelineState();
    } catch (e) {
      developer.log('getTimelineState failed: $e', name: 'EditorNotifier');
      return null;
    }
  }

  // ─── Effect Bridge Operations ──────────────────────────────────────

  /// Add a filter effect to the selected clip.
  Future<String?> addEffect(String filterTypeName) async {
    if (!_engineReady) return null;
    final clipId = state.selectedClipId;
    if (clipId == null) return null;
    try {
      final api = EngineService.instance.api;
      final effectInfo = await api.addEffect(
        clipId: clipId,
        filterTypeName: filterTypeName,
      );
      _refreshEngineState();
      return effectInfo.id;
    } catch (e) {
      developer.log('addEffect failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Add effect failed: $e');
      return null;
    }
  }

  /// Remove an effect from the selected clip.
  Future<void> removeEffect(String effectId) async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.removeEffect(clipId: clipId, effectId: effectId);
      _refreshEngineState();
    } catch (e) {
      developer.log('removeEffect failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Remove effect failed: $e');
    }
  }

  /// Set a parameter value for an effect on the selected clip.
  Future<void> setEffectParameter(
    String effectId,
    String paramName,
    double value,
  ) async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.setEffectParameter(
        clipId: clipId,
        effectId: effectId,
        paramName: paramName,
        value: value,
      );
      _refreshEngineState();
    } catch (e) {
      developer.log('setEffectParameter failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Set parameter failed: $e');
    }
  }

  /// Toggle the enabled/disabled state of an effect.
  Future<void> toggleEffect(String effectId) async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.toggleEffect(clipId: clipId, effectId: effectId);
      _refreshEngineState();
    } catch (e) {
      developer.log('toggleEffect failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Toggle effect failed: $e');
    }
  }

  /// Get the filter catalog from the engine.
  Future<List<dynamic>> getFilterCatalog() async {
    if (!_engineReady) return [];
    try {
      final api = EngineService.instance.api;
      return await api.getFilterCatalog();
    } catch (e) {
      developer.log('getFilterCatalog failed: $e', name: 'EditorNotifier');
      return [];
    }
  }

  /// Get the filter presets from the engine.
  Future<List<dynamic>> getFilterPresets() async {
    if (!_engineReady) return [];
    try {
      final api = EngineService.instance.api;
      return await api.getFilterPresets();
    } catch (e) {
      developer.log('getFilterPresets failed: $e', name: 'EditorNotifier');
      return [];
    }
  }

  /// Apply a filter preset to the selected clip.
  Future<void> applyFilterPreset(String presetId) async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.applyFilterPreset(clipId: clipId, presetId: presetId);
      _refreshEngineState();
    } catch (e) {
      developer.log('applyFilterPreset failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Apply preset failed: $e');
    }
  }

  // ─── Transition Bridge Operations ──────────────────────────────────

  /// Add a transition to the selected clip.
  Future<String?> addTransition(
    String transitionType,
    int durationMs,
    String direction,
  ) async {
    if (!_engineReady) return null;
    final clipId = state.selectedClipId;
    if (clipId == null) return null;
    try {
      final api = EngineService.instance.api;
      final info = await api.addTransition(
        clipId: clipId,
        transitionType: transitionType,
        durationMs: BigInt.from(durationMs),
        direction: direction,
      );
      _refreshEngineState();
      return info.id;
    } catch (e) {
      developer.log('addTransition failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Add transition failed: $e');
      return null;
    }
  }

  /// Remove a transition from the selected clip.
  Future<void> removeTransition(String direction) async {
    if (!_engineReady) return;
    final clipId = state.selectedClipId;
    if (clipId == null) return;
    try {
      final api = EngineService.instance.api;
      await api.removeTransition(clipId: clipId, direction: direction);
      _refreshEngineState();
    } catch (e) {
      developer.log('removeTransition failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Remove transition failed: $e');
    }
  }

  /// Get the transition catalog from the engine.
  Future<List<dynamic>> getTransitionCatalog() async {
    if (!_engineReady) return [];
    try {
      final api = EngineService.instance.api;
      return await api.getTransitionCatalog();
    } catch (e) {
      developer.log('getTransitionCatalog failed: $e', name: 'EditorNotifier');
      return [];
    }
  }

  // ─── Internal helpers ────────────────────────────────────────────

  /// Read the timeline duration from the engine and update state.
  Future<void> _syncDurationFromEngine() async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      final duration = await api.getTimelineDuration();
      state = state.copyWith(durationMs: duration.toInt());
    } catch (e) {
      developer.log('syncDuration failed: $e', name: 'EditorNotifier');
    }
  }

  /// Invalidate the engine-state caches so that downstream providers
  /// (project info, timeline duration) re-read fresh data.
  void _refreshEngineState() {
    try {
      _ref.read(engineStateRefresherProvider.notifier).refresh();
    } catch (_) {
      // Provider may not be mounted yet — safe to ignore.
    }
  }
}

/// Provider for editor state
final editorProvider = StateNotifierProvider<EditorNotifier, EditorState>((ref) {
  return EditorNotifier(ref);
});

/// Provider for current playback time formatted
final playbackTimeProvider = Provider<String>((ref) {
  final timeMs = ref.watch(editorProvider.select((s) => s.currentTimeMs));
  return Duration(milliseconds: timeMs).formatted;
});

/// Provider for duration formatted
final durationTimeProvider = Provider<String>((ref) {
  final durationMs = ref.watch(editorProvider.select((s) => s.durationMs));
  return Duration(milliseconds: durationMs).formatted;
});

/// Provider for waveform data, keyed by asset ID
final waveformProvider = FutureProvider.family<List<double>, String>((ref, assetId) async {
  final notifier = ref.read(editorProvider.notifier);
  return notifier.getWaveform(assetId);
});
