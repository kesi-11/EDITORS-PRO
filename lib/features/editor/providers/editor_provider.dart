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
    );
  }
}

enum LeftPanelTab { media, effects, text }

/// Editor state notifier — mediates between the UI and the Rust engine.
class EditorNotifier extends StateNotifier<EditorState> {
  Timer? _playbackTimer;
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
    state = state.copyWith(isPlaying: false);
  }

  /// Start playback from current position using a Timer
  void _startPlayback() {
    _playbackTimer?.cancel();
    const tickMs = 33; // ~30fps
    _playbackTimer = Timer.periodic(const Duration(milliseconds: tickMs), (timer) {
      if (!state.isPlaying || state.currentTimeMs >= state.durationMs) {
        timer.cancel();
        _playbackTimer = null;
        state = state.copyWith(isPlaying: false);
        return;
      }
      seekTo(state.currentTimeMs + tickMs);
    });
  }

  /// Seek to a specific time position
  void seekTo(int timeMs) {
    state = state.copyWith(
      currentTimeMs: timeMs.clamp(0, state.durationMs),
    );
  }

  /// Set the timeline duration
  void setDuration(int durationMs) {
    state = state.copyWith(durationMs: durationMs);
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
    state = state.copyWith(selectedTrackId: trackId);
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

  // ─── Bridge-wired operations ─────────────────────────────────────

  /// Undo last action via the Rust engine.
  Future<void> undo() async {
    if (!_engineReady) return;
    try {
      final api = EngineService.instance.api;
      await api.undo();
      await _syncDurationFromEngine();
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
    } catch (e) {
      developer.log('deleteSelected failed: $e', name: 'EditorNotifier');
      state = state.copyWith(lastError: 'Delete failed: $e');
    }
  }

  /// Import a media file via the Rust engine.
  ///
  /// Returns the [MediaAssetInfo] DTO on success, or `null` on failure.
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
  ///
  /// Returns the [ClipInfo] DTO on success, or `null` on failure.
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

  /// Create a new project via the Rust engine.
  ///
  /// Returns the [ProjectInfo] DTO on success, or `null` on failure.
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
