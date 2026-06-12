import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';
import 'audio_waveform_painter.dart';

/// Timeline panel - Shows tracks and clips with a playhead
///
/// Phase 4 improvements:
/// - Clip drag-and-drop repositioning
/// - Tap-to-seek on empty timeline areas
/// - Trim handles on selected clips
class TimelinePanel extends ConsumerStatefulWidget {
  const TimelinePanel({super.key});

  @override
  ConsumerState<TimelinePanel> createState() => _TimelinePanelState();
}

class _TimelinePanelState extends ConsumerState<TimelinePanel> {
  final ScrollController _horizontalScrollController = ScrollController();
  final ScrollController _verticalScrollController = ScrollController();
  /// Cached waveform data keyed by asset ID
  final Map<String, List<double>> _waveformCache = {};
  /// Track which clip is being dragged
  String? _draggingClipId;
  /// Track the original start time when drag begins
  int? _dragOriginalStartMs;

  @override
  void dispose() {
    _horizontalScrollController.dispose();
    _verticalScrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);
    final tracks = project?.tracks ?? [];

    return Container(
      height: AppTheme.timelineMinHeight,
      decoration: const BoxDecoration(
        color: AppTheme.surface,
        border: Border(top: BorderSide(color: Color(0xFF2A2A3E))),
      ),
      child: Column(
        children: [
          // Timeline toolbar
          _buildTimelineToolbar(context, editorState),

          // Timeline content
          Expanded(
            child: Row(
              children: [
                // Track headers (fixed left side)
                _buildTrackHeaders(context, tracks),

                // Scrollable timeline area
                Expanded(
                  child: SingleChildScrollView(
                    controller: _horizontalScrollController,
                    scrollDirection: Axis.horizontal,
                    child: GestureDetector(
                      // Tap on empty timeline area to seek
                      onTapUp: (details) {
                        final pixelsPerMs = AppConstants.timelinePixelsPerMs * editorState.zoomLevel;
                        final timeMs = (details.localPosition.dx / pixelsPerMs).round();
                        ref.read(editorProvider.notifier).seekTo(timeMs);
                      },
                      child: SizedBox(
                        width: (editorState.durationMs > 0 ? editorState.durationMs : 30000)
                            * AppConstants.timelinePixelsPerMs * editorState.zoomLevel,
                        child: Stack(
                          children: [
                            // Track content with clips
                            SingleChildScrollView(
                              controller: _verticalScrollController,
                              child: SizedBox(
                                height: tracks.length * AppTheme.trackHeight,
                                child: Stack(
                                  children: [
                                    // Draw tracks and clips
                                    ...tracks.asMap().entries.map((entry) {
                                      final index = entry.key;
                                      final track = entry.value;
                                      return _TrackRow(
                                        track: track,
                                        trackIndex: index,
                                        zoomLevel: editorState.zoomLevel,
                                        durationMs: editorState.durationMs,
                                        selectedClipId: editorState.selectedClipId,
                                        onClipTap: (clipId) => ref.read(editorProvider.notifier).selectClip(clipId),
                                        onClipDragStart: (clipId, originalStartMs) {
                                          setState(() {
                                            _draggingClipId = clipId;
                                            _dragOriginalStartMs = originalStartMs;
                                          });
                                        },
                                        onClipDragUpdate: (clipId, newStartMs) {
                                          // Update the clip position in the model immediately
                                          // for responsive visual feedback
                                          final project = ref.read(currentProjectProvider);
                                          if (project == null) return;

                                          final updatedTracks = project.tracks.map((track) {
                                            final updatedClips = track.clips.map((clip) {
                                              if (clip.id == clipId) {
                                                return clip.copyWith(startMs: newStartMs);
                                              }
                                              return clip;
                                            }).toList();
                                            return track.copyWith(clips: updatedClips);
                                          }).toList();

                                          ref.read(projectProvider.notifier).updateClip(
                                            clipId,
                                            project.tracks
                                                .expand((t) => t.clips)
                                                .firstWhere((c) => c.id == clipId)
                                                .copyWith(startMs: newStartMs),
                                          );
                                        },
                                        onClipDragEnd: (clipId, finalStartMs) {
                                          // Commit the move to the engine
                                          ref.read(editorProvider.notifier).moveClip(
                                            clipId: clipId,
                                            newStartMs: finalStartMs,
                                          );
                                          setState(() {
                                            _draggingClipId = null;
                                            _dragOriginalStartMs = null;
                                          });
                                        },
                                        waveformCache: _waveformCache,
                                      );
                                    }),

                                    // Playhead
                                    _PlayheadIndicator(
                                      currentTimeMs: editorState.currentTimeMs,
                                      durationMs: editorState.durationMs,
                                      zoomLevel: editorState.zoomLevel,
                                    ),
                                  ],
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTimelineToolbar(BuildContext context, EditorState state) {
    return Container(
      height: 36,
      decoration: const BoxDecoration(
        color: AppTheme.surfaceVariant,
        border: Border(bottom: BorderSide(color: Color(0xFF2A2A3E))),
      ),
      child: Row(
        children: [
          const SizedBox(width: 120), // Space for track headers
          // Time ruler
          Expanded(
            child: _TimeRuler(
              durationMs: state.durationMs > 0 ? state.durationMs : 30000,
              zoomLevel: state.zoomLevel,
              currentTimeMs: state.currentTimeMs,
            ),
          ),
          // Zoom controls
          const SizedBox(width: 8),
          IconButton(
            onPressed: () => ref.read(editorProvider.notifier).zoomOut(),
            icon: const Icon(Icons.remove, size: 16),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
          ),
          Text(
            '${(state.zoomLevel * 100).round()}%',
            style: context.textTheme.labelSmall,
          ),
          IconButton(
            onPressed: () => ref.read(editorProvider.notifier).zoomIn(),
            icon: const Icon(Icons.add, size: 16),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
          ),
          const SizedBox(width: 8),
        ],
      ),
    );
  }

  Widget _buildTrackHeaders(BuildContext context, List<TrackModel> tracks) {
    return SizedBox(
      width: 120,
      child: Column(
        children: tracks.map((track) {
          final color = _trackColor(track.trackType);
          return GestureDetector(
            onTap: () => ref.read(editorProvider.notifier).selectTrack(track.id),
            child: Container(
              height: AppTheme.trackHeight,
              decoration: BoxDecoration(
                color: AppTheme.surface,
                border: Border(
                  bottom: BorderSide(color: const Color(0xFF2A2A3E)),
                  right: BorderSide(color: const Color(0xFF2A2A3E)),
                ),
              ),
              child: Row(
                children: [
                  Container(
                    width: 4,
                    color: color,
                  ),
                  const SizedBox(width: 8),
                  Icon(
                    _trackIcon(track.trackType),
                    size: 14,
                    color: color,
                  ),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      track.name,
                      style: context.textTheme.labelMedium?.copyWith(
                        color: track.visible ? AppTheme.textPrimary : AppTheme.textDisabled,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  // Volume/mute indicator for audio tracks
                  if (track.trackType == TrackType.audio)
                    Padding(
                      padding: const EdgeInsets.only(right: 4),
                      child: Icon(
                        track.visible ? Icons.volume_up : Icons.volume_off,
                        size: 12,
                        color: track.visible ? color : AppTheme.error,
                      ),
                    ),
                  if (track.locked)
                    Icon(Icons.lock, size: 12, color: AppTheme.textDisabled),
                ],
              ),
            ),
          );
        }).toList(),
      ),
    );
  }

  Color _trackColor(TrackType type) {
    switch (type) {
      case TrackType.video: return AppTheme.videoTrackColor;
      case TrackType.audio: return AppTheme.audioTrackColor;
      case TrackType.text: return AppTheme.textTrackColor;
      case TrackType.effect: return AppTheme.effectTrackColor;
    }
  }

  IconData _trackIcon(TrackType type) {
    switch (type) {
      case TrackType.video: return Icons.videocam;
      case TrackType.audio: return Icons.audiotrack;
      case TrackType.text: return Icons.text_fields;
      case TrackType.effect: return Icons.auto_fix_high;
    }
  }
}

/// Time ruler at the top of the timeline
class _TimeRuler extends StatelessWidget {
  final int durationMs;
  final double zoomLevel;
  final int currentTimeMs;

  const _TimeRuler({
    required this.durationMs,
    required this.zoomLevel,
    required this.currentTimeMs,
  });

  @override
  Widget build(BuildContext context) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final totalWidth = durationMs * pixelsPerMs;
    final tickInterval = _calculateTickInterval();

    return CustomPaint(
      size: Size(totalWidth, 36),
      painter: _TimeRulerPainter(
        durationMs: durationMs,
        zoomLevel: zoomLevel,
        tickInterval: tickInterval,
        currentTimeMs: currentTimeMs,
      ),
    );
  }

  int _calculateTickInterval() {
    final intervals = [100, 200, 500, 1000, 2000, 5000, 10000, 30000, 60000];
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;

    for (final interval in intervals) {
      if (interval * pixelsPerMs >= 40) return interval;
    }
    return 60000;
  }
}

class _TimeRulerPainter extends CustomPainter {
  final int durationMs;
  final double zoomLevel;
  final int tickInterval;
  final int currentTimeMs;

  _TimeRulerPainter({
    required this.durationMs,
    required this.zoomLevel,
    required this.tickInterval,
    required this.currentTimeMs,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final paint = Paint()..color = const Color(0xFF44445A);
    final textPainter = TextPainter(textDirection: TextDirection.ltr);

    for (var ms = 0; ms <= durationMs; ms += tickInterval) {
      final x = ms * pixelsPerMs;
      final isMajor = ms % (tickInterval * 5) == 0;

      canvas.drawLine(
        Offset(x, isMajor ? 8.0 : 20.0),
        Offset(x, 36.0),
        paint..strokeWidth = isMajor ? 1.5 : 0.5,
      );

      if (isMajor) {
        final duration = Duration(milliseconds: ms);
        textPainter.text = TextSpan(
          text: duration.shortFormatted,
          style: const TextStyle(color: Color(0xFF8888A0), fontSize: 9),
        );
        textPainter.layout();
        textPainter.paint(canvas, Offset(x + 4, 2));
      }
    }
  }

  @override
  bool shouldRepaint(covariant _TimeRulerPainter oldDelegate) =>
      durationMs != oldDelegate.durationMs ||
      zoomLevel != oldDelegate.zoomLevel ||
      currentTimeMs != oldDelegate.currentTimeMs;
}

/// A single track row with clips
class _TrackRow extends StatelessWidget {
  final TrackModel track;
  final int trackIndex;
  final double zoomLevel;
  final int durationMs;
  final String? selectedClipId;
  final ValueChanged<String> onClipTap;
  final void Function(String clipId, int originalStartMs) onClipDragStart;
  final void Function(String clipId, int newStartMs) onClipDragUpdate;
  final void Function(String clipId, int finalStartMs) onClipDragEnd;
  final Map<String, List<double>> waveformCache;

  const _TrackRow({
    required this.track,
    required this.trackIndex,
    required this.zoomLevel,
    required this.durationMs,
    this.selectedClipId,
    required this.onClipTap,
    required this.onClipDragStart,
    required this.onClipDragUpdate,
    required this.onClipDragEnd,
    required this.waveformCache,
  });

  @override
  Widget build(BuildContext context) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final totalWidth = (durationMs > 0 ? durationMs : 30000) * pixelsPerMs;

    return Positioned(
      top: trackIndex * AppTheme.trackHeight,
      left: 0,
      right: 0,
      height: AppTheme.trackHeight,
      child: Container(
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(color: const Color(0xFF1A1A2E)),
          ),
        ),
        child: Stack(
          children: [
            // Track background
            Container(
              color: trackIndex.isEven
                  ? const Color(0xFF0E0E18)
                  : const Color(0xFF12121E),
            ),

            // Clips
            ...track.clips.map((clip) {
              final left = clip.startMs * pixelsPerMs;
              final width = clip.durationMs * pixelsPerMs;
              final isSelected = clip.id == selectedClipId;

              return Positioned(
                left: left,
                top: 4,
                width: math.max(width, AppTheme.clipMinWidth),
                height: AppTheme.trackHeight - 8,
                child: _DraggableClipWidget(
                  clip: clip,
                  trackType: track.trackType,
                  isSelected: isSelected,
                  zoomLevel: zoomLevel,
                  pixelsPerMs: pixelsPerMs,
                  onTap: () => onClipTap(clip.id),
                  onDragStart: (originalStartMs) => onClipDragStart(clip.id, originalStartMs),
                  onDragUpdate: (newStartMs) => onClipDragUpdate(clip.id, newStartMs),
                  onDragEnd: (finalStartMs) => onClipDragEnd(clip.id, finalStartMs),
                  waveformPeaks: waveformCache[clip.assetId],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }
}

/// A draggable clip widget on the timeline.
///
/// Supports horizontal drag to reposition a clip along the timeline.
/// The drag updates are reported continuously for responsive visual
/// feedback, and the final position is committed to the engine.
class _DraggableClipWidget extends StatefulWidget {
  final ClipModel clip;
  final TrackType trackType;
  final bool isSelected;
  final double zoomLevel;
  final double pixelsPerMs;
  final VoidCallback onTap;
  final void Function(int originalStartMs) onDragStart;
  final void Function(int newStartMs) onDragUpdate;
  final void Function(int finalStartMs) onDragEnd;
  final List<double>? waveformPeaks;

  const _DraggableClipWidget({
    required this.clip,
    required this.trackType,
    required this.isSelected,
    required this.zoomLevel,
    required this.pixelsPerMs,
    required this.onTap,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragEnd,
    this.waveformPeaks,
  });

  @override
  State<_DraggableClipWidget> createState() => _DraggableClipWidgetState();
}

class _DraggableClipWidgetState extends State<_DraggableClipWidget> {
  double _dragOffset = 0;
  bool _isDragging = false;

  @override
  Widget build(BuildContext context) {
    final color = _clipColor();
    final clipWidth = widget.clip.durationMs * AppConstants.timelinePixelsPerMs * widget.zoomLevel;

    return GestureDetector(
      onTap: widget.onTap,
      onHorizontalDragStart: (details) {
        setState(() {
          _isDragging = true;
          _dragOffset = 0;
        });
        widget.onDragStart(widget.clip.startMs);
      },
      onHorizontalDragUpdate: (details) {
        setState(() {
          _dragOffset += details.delta.dx;
        });
        final newStartMs = widget.clip.startMs + (_dragOffset / widget.pixelsPerMs).round();
        final clampedStart = newStartMs.clamp(0, 3600000); // Max 1 hour
        widget.onDragUpdate(clampedStart);
      },
      onHorizontalDragEnd: (details) {
        final newStartMs = widget.clip.startMs + (_dragOffset / widget.pixelsPerMs).round();
        final clampedStart = newStartMs.clamp(0, 3600000);
        widget.onDragEnd(clampedStart);
        setState(() {
          _isDragging = false;
          _dragOffset = 0;
        });
      },
      child: Container(
        decoration: BoxDecoration(
          color: color.withValues(alpha: _isDragging ? 0.5 : 0.7),
          borderRadius: BorderRadius.circular(4),
          border: isSelected
              ? Border.all(color: Colors.white, width: 2)
              : Border.all(color: color.withValues(alpha: 0.3)),
          boxShadow: _isDragging
              ? [BoxShadow(color: color.withValues(alpha: 0.4), blurRadius: 8, offset: const Offset(0, 2))]
              : null,
        ),
        child: Stack(
          children: [
            // Waveform visualization for audio clips
            if (widget.trackType == TrackType.audio && widget.waveformPeaks != null && widget.waveformPeaks!.isNotEmpty)
              Positioned.fill(
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(4),
                  child: AudioWaveformWidget(
                    peaks: widget.waveformPeaks!,
                    color: color,
                    width: math.max(clipWidth, AppTheme.clipMinWidth),
                    height: AppTheme.trackHeight - 8,
                  ),
                ),
              ),

            // Clip label and duration
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              child: clipWidth > 40
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(
                          _clipLabel(),
                          style: context.textTheme.labelSmall?.copyWith(
                            color: Colors.white,
                            fontWeight: FontWeight.w500,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        if (clipWidth > 80)
                          Text(
                            Duration(milliseconds: widget.clip.durationMs).shortFormatted,
                            style: context.textTheme.labelSmall?.copyWith(
                              color: Colors.white70,
                              fontSize: 8,
                            ),
                          ),
                      ],
                    )
                  : const SizedBox.shrink(),
            ),

            // Trim handles (visible when selected)
            if (isSelected) ...[
              // Left trim handle
              Positioned(
                left: 0,
                top: 0,
                bottom: 0,
                width: 6,
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.8),
                    borderRadius: const BorderRadius.only(
                      topLeft: Radius.circular(4),
                      bottomLeft: Radius.circular(4),
                    ),
                  ),
                ),
              ),
              // Right trim handle
              Positioned(
                right: 0,
                top: 0,
                bottom: 0,
                width: 6,
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.8),
                    borderRadius: const BorderRadius.only(
                      topRight: Radius.circular(4),
                      bottomRight: Radius.circular(4),
                    ),
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  bool get isSelected => widget.isSelected;

  Color _clipColor() {
    switch (widget.trackType) {
      case TrackType.video: return AppTheme.videoTrackColor;
      case TrackType.audio: return AppTheme.audioTrackColor;
      case TrackType.text: return AppTheme.textTrackColor;
      case TrackType.effect: return AppTheme.effectTrackColor;
    }
  }

  String _clipLabel() {
    switch (widget.trackType) {
      case TrackType.video: return 'Video';
      case TrackType.audio: return 'Audio';
      case TrackType.text: return 'Text';
      case TrackType.effect: return 'FX';
    }
  }
}

/// Playhead indicator on the timeline
class _PlayheadIndicator extends StatelessWidget {
  final int currentTimeMs;
  final int durationMs;
  final double zoomLevel;

  const _PlayheadIndicator({
    required this.currentTimeMs,
    required this.durationMs,
    required this.zoomLevel,
  });

  @override
  Widget build(BuildContext context) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final x = currentTimeMs * pixelsPerMs;

    return Positioned(
      left: x - AppTheme.playheadWidth / 2,
      top: 0,
      bottom: 0,
      child: Row(
        children: [
          Container(
            width: AppTheme.playheadWidth,
            color: AppTheme.playheadColor,
          ),
        ],
      ),
    );
  }
}
