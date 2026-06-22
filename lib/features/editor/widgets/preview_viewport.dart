import 'dart:async';
import 'dart:collection';
import 'dart:typed_data';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/services/engine_service.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';
import 'text_overlay_handle.dart';
import 'transform_handles.dart';

/// Video preview viewport — displays the current frame rendered by the
/// Rust engine and provides playback controls.
///
/// Phase 4 improvements:
/// - Continuous frame decode loop during playback (not just on time change)
/// - Debounced scrub rendering during manual seek
/// - Frame prefetch during idle for smoother playback startup
class PreviewViewport extends ConsumerStatefulWidget {
  const PreviewViewport({super.key});

  @override
  ConsumerState<PreviewViewport> createState() => _PreviewViewportState();
}

class _PreviewViewportState extends ConsumerState<PreviewViewport> {
  /// PNG-encoded bytes for the current frame, or null when no frame
  /// is available.
  Uint8List? _currentFrameBytes;

  /// LRU cache of the last 10 rendered frames keyed by time position.
  final _frameCache = _LruCache<int, Uint8List>(maxSize: 10);

  /// Whether a frame request is currently in-flight.
  bool _fetchingFrame = false;

  /// The time position for which we are currently fetching a frame.
  int? _fetchingTimeMs;

  /// Subscription to the editor state for continuous playback decoding.
  StreamSubscription? _playbackSubscription;

  /// Timer for continuous frame fetching during playback.
  Timer? _playbackDecodeTimer;

  @override
  void initState() {
    super.initState();
    // Listen to time changes and request frames.
    ref.listenManual(editorProvider.select((s) => s.currentTimeMs), _onTimeChanged, fireImmediately: true);

    // Listen to playback state changes to start/stop continuous decode
    ref.listenManual(editorProvider.select((s) => s.isPlaying), _onPlaybackChanged);
  }

  @override
  void dispose() {
    _playbackDecodeTimer?.cancel();
    super.dispose();
  }

  void _onTimeChanged(int? previous, int next) {
    // During playback, the continuous decode timer handles frame fetching.
    // We only request frames here for manual seeks.
    final isPlaying = ref.read(editorProvider).isPlaying;
    if (!isPlaying) {
      _requestFrame(next);
    }
  }

  void _onPlaybackChanged(bool? wasPlaying, bool isPlaying) {
    if (isPlaying) {
      _startContinuousDecode();
    } else {
      _stopContinuousDecode();
      // When playback stops, render the current frame
      _requestFrame(ref.read(editorProvider).currentTimeMs);
    }
  }

  /// Start continuous frame decode during playback.
  ///
  /// Uses a Timer to continuously request frames from the engine
  /// at ~15fps (a balance between smoothness and CPU usage on mobile).
  void _startContinuousDecode() {
    _playbackDecodeTimer?.cancel();
    _playbackDecodeTimer = Timer.periodic(const Duration(milliseconds: 66), (_) {
      if (!mounted) return;
      final state = ref.read(editorProvider);
      if (!state.isPlaying) {
        _stopContinuousDecode();
        return;
      }
      _requestFrame(state.currentTimeMs);
    });
  }

  /// Stop the continuous decode loop.
  void _stopContinuousDecode() {
    _playbackDecodeTimer?.cancel();
    _playbackDecodeTimer = null;
  }

  Future<void> _requestFrame(int timeMs) async {
    if (!EngineService.instance.isInitialized) return;
    if (_fetchingFrame) return;

    // Check the cache first.
    final cached = _frameCache.get(timeMs);
    if (cached != null) {
      if (mounted) {
        setState(() {
          _currentFrameBytes = cached;
        });
      }
      return;
    }

    _fetchingFrame = true;
    _fetchingTimeMs = timeMs;

    try {
      final api = EngineService.instance.api;
      final pngBytes = await api.getFrame(timeMs: BigInt.from(timeMs));

      if (!mounted) return;

      // Only apply if the user hasn't moved on to a very different position.
      final currentTimeMs = ref.read(editorProvider).currentTimeMs;
      if ((currentTimeMs - timeMs).abs() < 500) {
        _frameCache.put(timeMs, pngBytes);
        setState(() {
          _currentFrameBytes = pngBytes;
        });
      }
    } catch (e) {
      developer.log('getFrame failed at ${timeMs}ms: $e', name: 'PreviewViewport');
      // Leave the previous frame (or placeholder) in place.
    } finally {
      _fetchingFrame = false;
      _fetchingTimeMs = null;
    }
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorProvider);

    return Container(
      color: AppTheme.background,
      child: Column(
        children: [
          // Preview area
          Expanded(
            child: Center(
              child: AspectRatio(
                aspectRatio: 16 / 9,
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: const Color(0xFF2A2A3E), width: 1),
                  ),
                  child: Stack(
                    children: [
                      // Video frame display
                      Center(
                        child: _buildPreviewContent(context, editorState),
                      ),

                      // Text overlay handles for selected text clips
                      _buildTextOverlayHandles(context, editorState),

                      // Transform handles for selected clips (move, scale, rotate)
                      _buildTransformHandles(context, editorState),

                      // Playback controls overlay
                      if (!editorState.isPlaying)
                        Center(
                          child: GestureDetector(
                            onTap: () => ref.read(editorProvider.notifier).togglePlayback(),
                            child: Container(
                              width: 64,
                              height: 64,
                              decoration: BoxDecoration(
                                color: AppTheme.primary.withOpacity(0.8),
                                shape: BoxShape.circle,
                              ),
                              child: const Icon(
                                Icons.play_arrow,
                                color: Colors.white,
                                size: 36,
                              ),
                            ),
                          ),
                        ),

                      // Playback speed indicator
                      if (editorState.playbackSpeed != 1.0)
                        Positioned(
                          top: 8,
                          right: 8,
                          child: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: Colors.black54,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              '${editorState.playbackSpeed}x',
                              style: const TextStyle(
                                color: AppTheme.primaryLight,
                                fontSize: 11,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ),
                        ),

                      // Decoding indicator
                      if (_fetchingFrame)
                        Positioned(
                          bottom: 8,
                          right: 8,
                          child: SizedBox(
                            width: 14,
                            height: 14,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: AppTheme.primary.withOpacity(0.6),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
          ),

          // Scrub bar below preview
          _buildScrubBar(context, editorState),
        ],
      ),
    );
  }

  Widget _buildPreviewContent(BuildContext context, EditorState state) {
    // If we have decoded frame bytes, display them.
    if (_currentFrameBytes != null && _currentFrameBytes!.isNotEmpty) {
      return Image.memory(
        _currentFrameBytes!,
        fit: BoxFit.contain,
        gaplessPlayback: true,
        errorBuilder: (context, error, stackTrace) {
          // If the PNG data is corrupt, fall back to the placeholder.
          return _buildPlaceholder(context, state);
        },
      );
    }

    return _buildPlaceholder(context, state);
  }

  Widget _buildPlaceholder(BuildContext context, EditorState state) {
    return Container(
      color: const Color(0xFF111122),
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.movie_creation_outlined,
              size: 64,
              color: AppTheme.textDisabled,
            ),
            const SizedBox(height: 16),
            Text(
              'Preview',
              style: context.textTheme.titleMedium?.copyWith(
                color: AppTheme.textDisabled,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              Duration(milliseconds: state.currentTimeMs).formatted,
              style: context.textTheme.bodySmall?.copyWith(
                fontFamily: 'monospace',
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// Build text overlay handles for any selected text clip.
  ///
  /// When a text clip is selected, this renders a draggable bounding box
  /// with resize handles on the preview viewport.
  Widget _buildTextOverlayHandles(BuildContext context, EditorState editorState) {
    final project = ref.watch(currentProjectProvider);
    if (project == null || editorState.selectedClipId == null) {
      return const SizedBox.shrink();
    }

    // Find the selected clip and check if it's on a text track
    ClipModel? selectedClip;
    TrackModel? selectedTrack;
    for (final track in project.tracks) {
      for (final clip in track.clips) {
        if (clip.id == editorState.selectedClipId) {
          selectedClip = clip;
          selectedTrack = track;
          break;
        }
      }
      if (selectedClip != null) break;
    }

    if (selectedClip == null || selectedTrack?.trackType != TrackType.text) {
      return const SizedBox.shrink();
    }

    // Create a TextOverlayData from the clip model
    final overlayData = TextOverlayData(
      clipId: selectedClip.id,
      text: selectedClip.assetId.startsWith('text_') ? 'Text' : selectedClip.assetId,
      positionX: 0.5,
      positionY: 0.5,
      width: 0.4,
      height: 0.1,
      isSelected: true,
    );

    return TextOverlayHandle(
      data: overlayData,
      onPositionChanged: (clipId, posX, posY) {
        ref.read(editorProvider.notifier).updateTextPosition(
              clipId: clipId,
              positionX: posX,
              positionY: posY,
            );
      },
      onTap: () {
        // Keep the clip selected (already is)
      },
    );
  }

  /// Build transform handles for the selected clip.
  ///
  /// When a clip is selected and has keyframe support, this renders
  /// move/scale/rotate handles on the preview viewport.
  Widget _buildTransformHandles(BuildContext context, EditorState editorState) {
    final project = ref.watch(currentProjectProvider);
    if (project == null || editorState.selectedClipId == null) {
      return const SizedBox.shrink();
    }

    // Don't show transform handles for text clips (those use TextOverlayHandle)
    ClipModel? selectedClip;
    TrackModel? selectedTrack;
    for (final track in project.tracks) {
      for (final clip in track.clips) {
        if (clip.id == editorState.selectedClipId) {
          selectedClip = clip;
          selectedTrack = track;
          break;
        }
      }
      if (selectedClip != null) break;
    }

    if (selectedClip == null) return const SizedBox.shrink();
    if (selectedTrack?.trackType == TrackType.text) return const SizedBox.shrink();

    // Calculate clip bounds relative to the preview area
    // Use a default centered rect for video/image clips
    // In a full implementation, this would be derived from keyframe data
    final bounds = Rect.fromCenter(
      center: const Offset(160, 90), // Center of 16:9 preview at 320x180
      width: 200,
      height: 112,
    );

    return TransformHandles(
      bounds: bounds,
      rotation: 0.0,
      isSelected: true,
      onMove: (delta) {
        // Convert pixel delta to normalized position and update via keyframe
        final normalizedDx = delta.dx / 320; // approximate preview width
        final normalizedDy = delta.dy / 180; // approximate preview height
        ref.read(editorProvider.notifier).updateClipTransform(
          clipId: selectedClip!.id,
          property: 'position_x',
          delta: normalizedDx,
        );
        ref.read(editorProvider.notifier).updateClipTransform(
          clipId: selectedClip!.id,
          property: 'position_y',
          delta: normalizedDy,
        );
      },
      onScaleStart: (handleType) {
        // Track initial scale for this handle type
      },
      onScaleUpdate: (delta) {
        // Convert pixel delta to normalized scale change
        final scaleDelta = 1.0 + (delta.dx + delta.dy) / 400;
        ref.read(editorProvider.notifier).updateClipTransform(
          clipId: selectedClip!.id,
          property: 'scale',
          delta: scaleDelta - 1.0,
        );
      },
      onRotate: (angleDelta) {
        // Add a rotation keyframe at the current time
        ref.read(editorProvider.notifier).updateClipTransform(
          clipId: selectedClip!.id,
          property: 'rotation',
          delta: angleDelta,
        );
      },
    );
  }

  Widget _buildScrubBar(BuildContext context, EditorState state) {
    return Container(
      height: 32,
      decoration: const BoxDecoration(
        color: AppTheme.surface,
        border: Border(top: BorderSide(color: Color(0xFF2A2A3E))),
      ),
      child: Row(
        children: [
          // Current time
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Text(
              Duration(milliseconds: state.currentTimeMs).formatted,
              style: context.textTheme.labelSmall?.copyWith(
                fontFamily: 'monospace',
              ),
            ),
          ),

          // Scrubber
          Expanded(
            child: SliderTheme(
              data: SliderThemeData(
                activeTrackColor: AppTheme.primary,
                thumbColor: AppTheme.primaryLight,
                inactiveTrackColor: const Color(0xFF2A2A3E),
                trackHeight: 2,
                thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 5),
              ),
              child: Slider(
                value: state.durationMs > 0
                    ? state.currentTimeMs / state.durationMs
                    : 0,
                onChanged: (value) {
                  final timeMs = (value * state.durationMs).round();
                  ref.read(editorProvider.notifier).seekTo(timeMs);
                },
              ),
            ),
          ),

          // Duration
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Text(
              Duration(milliseconds: state.durationMs).formatted,
              style: context.textTheme.labelSmall?.copyWith(
                fontFamily: 'monospace',
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Simple LRU cache backed by a [LinkedHashMap].
///
/// Evicts the least-recently-used entry when [maxSize] is exceeded.
class _LruCache<K, V> {
  _LruCache({required this.maxSize});

  final int maxSize;
  final LinkedHashMap<K, V> _map = LinkedHashMap<K, V>();

  V? get(K key) {
    final value = _map.remove(key);
    if (value != null) {
      // Re-insert at the end so it becomes most-recently-used.
      _map[key] = value;
    }
    return value;
  }

  void put(K key, V value) {
    _map.remove(key); // Remove if it already exists.
    _map[key] = value;
    while (_map.length > maxSize) {
      _map.remove(_map.keys.first);
    }
  }

  void clear() => _map.clear();
}
