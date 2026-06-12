import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../data/models/project_model.dart';

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
    );
  }
}

enum LeftPanelTab { media, effects, text }

/// Editor state notifier
class EditorNotifier extends StateNotifier<EditorState> {
  Timer? _playbackTimer;

  EditorNotifier() : super(const EditorState());

  @override
  void dispose() {
    _playbackTimer?.cancel();
    super.dispose();
  }

  /// Initialize the editor
  void initialize() {
    // Set up initial state, connect to Rust engine, etc.
    state = const EditorState();
  }

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

  /// Undo last action
  void undo() {
    // Will call Rust engine's undo
  }

  /// Redo last action
  void redo() {
    // Will call Rust engine's redo
  }

  /// Split the currently selected clip at the playhead
  void splitAtPlayhead() {
    if (state.selectedClipId != null) {
      // Will call Rust engine's split_clip
    }
  }

  /// Delete the currently selected clip
  void deleteSelected() {
    if (state.selectedClipId != null) {
      // Will call Rust engine's remove_clip
      state = state.copyWith(selectedClipId: null, showInspector: false);
    }
  }
}

/// Provider for editor state
final editorProvider = StateNotifierProvider<EditorNotifier, EditorState>((ref) {
  return EditorNotifier();
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
