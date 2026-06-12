import 'dart:async';
import 'dart:collection';
import 'dart:typed_data';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/services/engine_service.dart';
import '../providers/editor_provider.dart';

/// Video preview viewport — displays the current frame rendered by the
/// Rust engine and provides playback controls.
class PreviewViewport extends ConsumerStatefulWidget {
  const PreviewViewport({super.key});

  @override
  ConsumerState<PreviewViewport> createState() => _PreviewViewportState();
}

class _PreviewViewportState extends ConsumerState<PreviewViewport> {
  /// PNG-encoded bytes for the current frame, or null when no frame
  /// is available.
  Uint8List? _currentFrameBytes;

  /// LRU cache of the last 5 rendered frames keyed by time position.
  final _frameCache = _LruCache<int, Uint8List>(maxSize: 5);

  /// Whether a frame request is currently in-flight.
  bool _fetchingFrame = false;

  /// The time position for which we are currently fetching a frame.
  int? _fetchingTimeMs;

  @override
  void initState() {
    super.initState();
    // Listen to time changes and request frames.
    ref.listenManual(editorProvider.select((s) => s.currentTimeMs), _onTimeChanged, fireImmediately: true);
  }

  void _onTimeChanged(int? previous, int next) {
    _requestFrame(next);
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

                      // Playback controls overlay
                      if (!editorState.isPlaying)
                        Center(
                          child: GestureDetector(
                            onTap: () => ref.read(editorProvider.notifier).togglePlayback(),
                            child: Container(
                              width: 64,
                              height: 64,
                              decoration: BoxDecoration(
                                color: AppTheme.primary.withValues(alpha: 0.8),
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
