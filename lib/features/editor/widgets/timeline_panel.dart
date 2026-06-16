import 'dart:collection';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/constants/app_icons.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/app_icon.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';
import 'audio_waveform_painter.dart';

/// Width of the fixed track-header column on the left of the timeline.
const double _kTrackHeaderWidth = 120;

/// Height of the timeline toolbar (ruler + zoom controls).
const double _kToolbarHeight = 40;

/// Maximum clip start position (1 hour) used when clamping drags.
const int _kMaxClipStartMs = 3600000;

/// Timeline panel — shows tracks and clips with a playhead.
///
/// Designed to feel like DaVinci Resolve / Premiere Pro:
/// - A compact toolbar with a synced time ruler and zoom controls.
/// - Fixed track headers with type icon, name, and per-track toggles
///   (visibility, lock, and mute/solo for audio).
/// - A scrollable track area with vibrant gradient clips, trim handles,
///   drag feedback, and a red playhead with a triangle handle.
class TimelinePanel extends ConsumerStatefulWidget {
  const TimelinePanel({super.key});

  @override
  ConsumerState<TimelinePanel> createState() => _TimelinePanelState();
}

class _TimelinePanelState extends ConsumerState<TimelinePanel> {
  final ScrollController _horizontalScrollController = ScrollController();
  final ScrollController _verticalScrollController = ScrollController();

  /// Cached waveform data keyed by asset ID (bounded LRU cache).
  final _LruCache<String, List<double>> _waveformCache =
      _LruCache<String, List<double>>(maxSize: 32);

  /// Track IDs that are soloed for the current session (visual indicator).
  final Set<String> _soloedTrackIds = <String>{};

  @override
  void dispose() {
    _horizontalScrollController.dispose();
    _verticalScrollController.dispose();
    super.dispose();
  }

  void _toggleVisibility(TrackModel track) {
    ref
        .read(projectProvider.notifier)
        .updateTrack(track.id, track.copyWith(visible: !track.visible));
  }

  void _toggleLock(TrackModel track) {
    ref
        .read(projectProvider.notifier)
        .updateTrack(track.id, track.copyWith(locked: !track.locked));
  }

  void _toggleMute(TrackModel track) {
    final muted = track.volume <= 0;
    ref.read(projectProvider.notifier).updateTrack(
          track.id,
          track.copyWith(volume: muted ? 1.0 : 0.0),
        );
  }

  void _toggleSolo(String trackId) {
    setState(() {
      if (_soloedTrackIds.contains(trackId)) {
        _soloedTrackIds.remove(trackId);
      } else {
        _soloedTrackIds.add(trackId);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);
    final tracks = project?.tracks ?? <TrackModel>[];

    final pixelsPerMs =
        AppConstants.timelinePixelsPerMs * editorState.zoomLevel;
    final duration = editorState.durationMs > 0 ? editorState.durationMs : 30000;
    final totalWidth = duration * pixelsPerMs;
    final tracksHeight = tracks.length * AppTheme.trackHeight;

    final assetNames = <String, String>{
      for (final asset in project?.mediaAssets ?? <MediaAssetModel>[])
        asset.id: asset.fileName,
    };

    return Container(
      height: AppTheme.timelineMinHeight,
      decoration: const BoxDecoration(
        color: AppTheme.background,
        border: Border(top: BorderSide(color: AppTheme.border)),
      ),
      child: Column(
        children: [
          _buildTimelineToolbar(context, editorState, duration, totalWidth),
          Expanded(
            child: Stack(
              children: [
                // Vertical scroll wraps headers + tracks so they stay aligned.
                SingleChildScrollView(
                  controller: _verticalScrollController,
                  child: SizedBox(
                    height: tracksHeight,
                    child: Row(
                      children: [
                        _buildTrackHeaders(tracks),
                        Expanded(
                          child: SingleChildScrollView(
                            controller: _horizontalScrollController,
                            scrollDirection: Axis.horizontal,
                            child: GestureDetector(
                              behavior: HitTestBehavior.opaque,
                              onTapUp: (details) {
                                final timeMs =
                                    (details.localPosition.dx / pixelsPerMs)
                                        .round()
                                        .clamp(0, duration);
                                ref
                                    .read(editorProvider.notifier)
                                    .seekTo(timeMs);
                              },
                              child: SizedBox(
                                width: totalWidth,
                                child: Stack(
                                  children: tracks.asMap().entries.map((entry) {
                                    final index = entry.key;
                                    final track = entry.value;
                                    return _TrackRow(
                                      track: track,
                                      trackIndex: index,
                                      zoomLevel: editorState.zoomLevel,
                                      durationMs: duration,
                                      selectedClipId:
                                          editorState.selectedClipId,
                                      assetNames: assetNames,
                                      waveformCache: _waveformCache,
                                      onClipTap: (clipId) => ref
                                          .read(editorProvider.notifier)
                                          .selectClip(clipId),
                                      onClipDragUpdate: (clipId, newStartMs) {
                                        final current =
                                            ref.read(currentProjectProvider);
                                        if (current == null) return;
                                        ref
                                            .read(projectProvider.notifier)
                                            .updateClip(
                                              clipId,
                                              current.tracks
                                                  .expand((t) => t.clips)
                                                  .firstWhere(
                                                    (c) => c.id == clipId,
                                                  )
                                                  .copyWith(
                                                    startMs: newStartMs,
                                                  ),
                                            );
                                      },
                                      onClipDragEnd: (clipId, finalStartMs) {
                                        ref
                                            .read(editorProvider.notifier)
                                            .moveClip(
                                              clipId: clipId,
                                              newStartMs: finalStartMs,
                                            );
                                      },
                                    );
                                  }).toList(),
                                ),
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                // Playhead overlay — spans the visible track area only.
                Positioned(
                  left: _kTrackHeaderWidth,
                  top: 0,
                  right: 0,
                  bottom: 0,
                  child: ClipRect(
                    child: IgnorePointer(
                      child: _PlayheadIndicator(
                        currentTimeMs: editorState.currentTimeMs,
                        pixelsPerMs: pixelsPerMs,
                        scrollController: _horizontalScrollController,
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

  Widget _buildTimelineToolbar(
    BuildContext context,
    EditorState state,
    int duration,
    double totalWidth,
  ) {
    return Container(
      height: _kToolbarHeight,
      decoration: const BoxDecoration(
        color: AppTheme.surfaceVariant,
        border: Border(bottom: BorderSide(color: AppTheme.border)),
      ),
      child: Row(
        children: [
          const SizedBox(width: _kTrackHeaderWidth),
          Expanded(
            child: ClipRect(
              child: AnimatedBuilder(
                animation: _horizontalScrollController,
                builder: (context, _) {
                  final offset = _horizontalScrollController.hasClients
                      ? _horizontalScrollController.offset
                      : 0.0;
                  return Stack(
                    children: [
                      Positioned(
                        left: -offset,
                        top: 0,
                        bottom: 0,
                        width: totalWidth,
                        child: _TimeRuler(
                          durationMs: duration,
                          zoomLevel: state.zoomLevel,
                          currentTimeMs: state.currentTimeMs,
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          ),
          const SizedBox(width: 8),
          _TimelineIconButton(
            icon: AppIcons.zoomOut,
            size: 16,
            onTap: () => ref.read(editorProvider.notifier).zoomOut(),
          ),
          const SizedBox(width: 4),
          SizedBox(
            width: 46,
            child: Text(
              '${(state.zoomLevel * 100).round()}%',
              textAlign: TextAlign.center,
              style: context.textTheme.labelSmall?.copyWith(
                fontFamily: 'monospace',
                color: AppTheme.textSecondary,
              ),
            ),
          ),
          const SizedBox(width: 4),
          _TimelineIconButton(
            icon: AppIcons.zoomIn,
            size: 16,
            onTap: () => ref.read(editorProvider.notifier).zoomIn(),
          ),
          const SizedBox(width: 8),
        ],
      ),
    );
  }

  Widget _buildTrackHeaders(List<TrackModel> tracks) {
    return SizedBox(
      width: _kTrackHeaderWidth,
      child: Column(
        children: tracks.asMap().entries.map((entry) {
          final index = entry.key;
          final track = entry.value;
          return _TrackHeaderItem(
            track: track,
            trackIndex: index,
            color: _trackColor(track.trackType),
            isSoloed: _soloedTrackIds.contains(track.id),
            onSelect: () =>
                ref.read(editorProvider.notifier).selectTrack(track.id),
            onToggleVisibility: () => _toggleVisibility(track),
            onToggleLock: () => _toggleLock(track),
            onToggleMute: () => _toggleMute(track),
            onToggleSolo: () => _toggleSolo(track.id),
          );
        }).toList(),
      ),
    );
  }
}

// ─── Top-level helpers ────────────────────────────────────────────────

Color _trackColor(TrackType type) {
  switch (type) {
    case TrackType.video:
      return AppTheme.videoTrackColor;
    case TrackType.audio:
      return AppTheme.audioTrackColor;
    case TrackType.text:
      return AppTheme.textTrackColor;
    case TrackType.effect:
      return AppTheme.effectTrackColor;
  }
}

String _trackIconPath(TrackType type) {
  switch (type) {
    case TrackType.video:
      return AppIcons.video;
    case TrackType.audio:
      return AppIcons.audio;
    case TrackType.text:
      return AppIcons.text;
    case TrackType.effect:
      return AppIcons.effects;
  }
}

/// Time ruler at the top of the timeline.
class _TimeRuler extends StatelessWidget {
  const _TimeRuler({
    required this.durationMs,
    required this.zoomLevel,
    required this.currentTimeMs,
  });

  final int durationMs;
  final double zoomLevel;
  final int currentTimeMs;

  @override
  Widget build(BuildContext context) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final totalWidth = durationMs * pixelsPerMs;
    final tickInterval = _calculateTickInterval();

    return CustomPaint(
      size: Size(totalWidth, _kToolbarHeight),
      painter: _TimeRulerPainter(
        durationMs: durationMs,
        zoomLevel: zoomLevel,
        tickInterval: tickInterval,
        currentTimeMs: currentTimeMs,
      ),
    );
  }

  int _calculateTickInterval() {
    const intervals = [100, 200, 500, 1000, 2000, 5000, 10000, 30000, 60000];
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    for (final interval in intervals) {
      if (interval * pixelsPerMs >= 40) return interval;
    }
    return 60000;
  }
}

class _TimeRulerPainter extends CustomPainter {
  const _TimeRulerPainter({
    required this.durationMs,
    required this.zoomLevel,
    required this.tickInterval,
    required this.currentTimeMs,
  });

  final int durationMs;
  final double zoomLevel;
  final int tickInterval;
  final int currentTimeMs;

  @override
  void paint(Canvas canvas, Size size) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;

    // Background wash.
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = AppTheme.surfaceVariant,
    );

    // Baseline separator.
    canvas.drawLine(
      Offset(0, size.height - 1),
      Offset(size.width, size.height - 1),
      Paint()
        ..color = AppTheme.border
        ..strokeWidth = 1,
    );

    final majorPaint = Paint()
      ..color = AppTheme.borderLight
      ..strokeWidth = 1;
    final minorPaint = Paint()
      ..color = AppTheme.border.withValues(alpha: 0.7)
      ..strokeWidth = 1;
    final textPainter = TextPainter(textDirection: TextDirection.ltr);

    for (var ms = 0; ms <= durationMs; ms += tickInterval) {
      final x = ms * pixelsPerMs;
      final isMajor = ms % (tickInterval * 5) == 0;
      final tickTop = isMajor ? size.height * 0.45 : size.height * 0.7;
      canvas.drawLine(
        Offset(x, tickTop),
        Offset(x, size.height - 1),
        isMajor ? majorPaint : minorPaint,
      );

      if (isMajor) {
        textPainter.text = TextSpan(
          text: Duration(milliseconds: ms).shortFormatted,
          style: const TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 9,
            fontFamily: 'monospace',
            fontWeight: FontWeight.w500,
          ),
        );
        textPainter.layout();
        textPainter.paint(canvas, Offset(x + 4, 3));
      }
    }

    // Playhead marker inside the ruler.
    if (currentTimeMs >= 0 && currentTimeMs <= durationMs) {
      final px = currentTimeMs * pixelsPerMs;
      canvas.drawLine(
        Offset(px, 0),
        Offset(px, size.height),
        Paint()
          ..color = AppTheme.playheadColor
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _TimeRulerPainter oldDelegate) =>
      durationMs != oldDelegate.durationMs ||
      zoomLevel != oldDelegate.zoomLevel ||
      currentTimeMs != oldDelegate.currentTimeMs ||
      tickInterval != oldDelegate.tickInterval;
}

/// A single track row with its clips.
class _TrackRow extends StatelessWidget {
  const _TrackRow({
    required this.track,
    required this.trackIndex,
    required this.zoomLevel,
    required this.durationMs,
    required this.selectedClipId,
    required this.assetNames,
    required this.waveformCache,
    required this.onClipTap,
    required this.onClipDragUpdate,
    required this.onClipDragEnd,
  });

  final TrackModel track;
  final int trackIndex;
  final double zoomLevel;
  final int durationMs;
  final String? selectedClipId;
  final Map<String, String> assetNames;
  final _LruCache<String, List<double>> waveformCache;
  final ValueChanged<String> onClipTap;
  final void Function(String clipId, int newStartMs) onClipDragUpdate;
  final void Function(String clipId, int finalStartMs) onClipDragEnd;

  @override
  Widget build(BuildContext context) {
    final pixelsPerMs = AppConstants.timelinePixelsPerMs * zoomLevel;
    final isEven = trackIndex.isEven;

    return Positioned(
      top: trackIndex * AppTheme.trackHeight,
      left: 0,
      right: 0,
      height: AppTheme.trackHeight,
      child: Container(
        decoration: BoxDecoration(
          color: isEven ? AppTheme.surface : AppTheme.surfaceVariant,
          border: Border(
            bottom: BorderSide(
              color: AppTheme.border.withValues(alpha: 0.6),
            ),
          ),
        ),
        child: Stack(
          children: track.clips.map((clip) {
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
                clipName: assetNames[clip.assetId],
                isSelected: isSelected,
                zoomLevel: zoomLevel,
                pixelsPerMs: pixelsPerMs,
                onTap: () => onClipTap(clip.id),
                onDragUpdate: (newStartMs) =>
                    onClipDragUpdate(clip.id, newStartMs),
                onDragEnd: (finalStartMs) =>
                    onClipDragEnd(clip.id, finalStartMs),
                waveformPeaks: waveformCache.get(clip.assetId),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }
}

/// A draggable clip widget on the timeline.
///
/// Supports horizontal drag to reposition a clip. The drag offset is
/// reported continuously (via [onDragUpdate]) for responsive feedback,
/// and the final position is committed to the engine (via [onDragEnd]).
class _DraggableClipWidget extends StatefulWidget {
  const _DraggableClipWidget({
    required this.clip,
    required this.trackType,
    required this.clipName,
    required this.isSelected,
    required this.zoomLevel,
    required this.pixelsPerMs,
    required this.onTap,
    required this.onDragUpdate,
    required this.onDragEnd,
    this.waveformPeaks,
  });

  final ClipModel clip;
  final TrackType trackType;
  final String? clipName;
  final bool isSelected;
  final double zoomLevel;
  final double pixelsPerMs;
  final VoidCallback onTap;
  final void Function(int newStartMs) onDragUpdate;
  final void Function(int finalStartMs) onDragEnd;
  final List<double>? waveformPeaks;

  @override
  State<_DraggableClipWidget> createState() => _DraggableClipWidgetState();
}

class _DraggableClipWidgetState extends State<_DraggableClipWidget> {
  double _dragOffset = 0;
  int _dragStartMs = 0;
  bool _isDragging = false;

  @override
  Widget build(BuildContext context) {
    final baseColor = _clipColor();
    final lightColor = _clipLightColor();
    final clipWidth = widget.clip.durationMs *
        AppConstants.timelinePixelsPerMs *
        widget.zoomLevel;
    final isAudio = widget.trackType == TrackType.audio;

    return GestureDetector(
      onTap: widget.onTap,
      onHorizontalDragStart: (_) {
        setState(() {
          _isDragging = true;
          _dragOffset = 0;
          _dragStartMs = widget.clip.startMs;
        });
      },
      onHorizontalDragUpdate: (details) {
        setState(() {
          _dragOffset += details.delta.dx;
        });
        final newStart = (_dragStartMs + (_dragOffset / widget.pixelsPerMs))
            .round()
            .clamp(0, _kMaxClipStartMs);
        widget.onDragUpdate(newStart);
      },
      onHorizontalDragEnd: (_) {
        final newStart = (_dragStartMs + (_dragOffset / widget.pixelsPerMs))
            .round()
            .clamp(0, _kMaxClipStartMs);
        widget.onDragEnd(newStart);
        setState(() {
          _isDragging = false;
          _dragOffset = 0;
        });
      },
      child: Opacity(
        opacity: _isDragging ? 0.55 : 1.0,
        child: Container(
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topCenter,
              end: Alignment.bottomCenter,
              colors: [
                lightColor.withValues(alpha: 0.95),
                baseColor.withValues(alpha: 0.55),
              ],
            ),
            borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
            border: Border(
              left: BorderSide(color: baseColor, width: 3),
              top: BorderSide(
                color: baseColor.withValues(alpha: 0.6),
                width: 1,
              ),
              right: BorderSide(
                color: baseColor.withValues(alpha: 0.6),
                width: 1,
              ),
              bottom: BorderSide(
                color: baseColor.withValues(alpha: 0.85),
                width: 1,
              ),
            ),
            boxShadow: _boxShadows(baseColor),
          ),
          child: Stack(
            children: [
              // Waveform visualization for audio clips.
              if (isAudio &&
                  widget.waveformPeaks != null &&
                  widget.waveformPeaks!.isNotEmpty)
                Positioned.fill(
                  child: ClipRRect(
                    borderRadius:
                        BorderRadius.circular(AppTheme.radiusSmall),
                    child: AudioWaveformWidget(
                      peaks: widget.waveformPeaks!,
                      color: Colors.white.withValues(alpha: 0.85),
                      width: math.max(clipWidth, AppTheme.clipMinWidth),
                      height: AppTheme.trackHeight - 8,
                    ),
                  ),
                ),
              // Label + duration.
              Positioned(
                left: 8,
                right: 8,
                top: 4,
                bottom: 4,
                child: clipWidth > 40
                    ? Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Text(
                            _clipLabel(),
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                              height: 1.1,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          if (clipWidth > 70)
                            Padding(
                              padding: const EdgeInsets.only(top: 1),
                              child: Text(
                                Duration(milliseconds: widget.clip.durationMs)
                                    .shortFormatted,
                                style: TextStyle(
                                  color: Colors.white.withValues(alpha: 0.7),
                                  fontSize: 8,
                                  fontFamily: 'monospace',
                                  height: 1.1,
                                ),
                              ),
                            ),
                        ],
                      )
                    : const SizedBox.shrink(),
              ),
              // Selection border overlay.
              if (widget.isSelected)
                Positioned.fill(
                  child: IgnorePointer(
                    child: DecoratedBox(
                      decoration: BoxDecoration(
                        borderRadius:
                            BorderRadius.circular(AppTheme.radiusSmall),
                        border: Border.all(color: Colors.white, width: 2),
                      ),
                    ),
                  ),
                ),
              // Trim handles.
              if (widget.isSelected) ...[
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  width: 6,
                  child: _TrimHandle(left: true, color: baseColor),
                ),
                Positioned(
                  right: 0,
                  top: 0,
                  bottom: 0,
                  width: 6,
                  child: _TrimHandle(left: false, color: baseColor),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  List<BoxShadow> _boxShadows(Color baseColor) {
    if (_isDragging) {
      return [
        BoxShadow(
          color: baseColor.withValues(alpha: 0.6),
          blurRadius: 14,
          offset: const Offset(0, 3),
        ),
      ];
    }
    if (widget.isSelected) {
      return [
        BoxShadow(
          color: Colors.white.withValues(alpha: 0.18),
          blurRadius: 10,
        ),
      ];
    }
    return [
      BoxShadow(
        color: Colors.black.withValues(alpha: 0.25),
        blurRadius: 4,
        offset: const Offset(0, 2),
      ),
    ];
  }

  Color _clipColor() {
    switch (widget.trackType) {
      case TrackType.video:
        return AppTheme.videoTrackColor;
      case TrackType.audio:
        return AppTheme.audioTrackColor;
      case TrackType.text:
        return AppTheme.textTrackColor;
      case TrackType.effect:
        return AppTheme.effectTrackColor;
    }
  }

  Color _clipLightColor() {
    switch (widget.trackType) {
      case TrackType.video:
        return AppTheme.videoTrackColorLight;
      case TrackType.audio:
        return AppTheme.audioTrackColorLight;
      case TrackType.text:
        return Color.lerp(AppTheme.textTrackColor, Colors.white, 0.3)!;
      case TrackType.effect:
        return Color.lerp(AppTheme.effectTrackColor, Colors.white, 0.3)!;
    }
  }

  String _clipLabel() {
    if (widget.clipName != null && widget.clipName!.isNotEmpty) {
      return widget.clipName!;
    }
    switch (widget.trackType) {
      case TrackType.video:
        return 'Video';
      case TrackType.audio:
        return 'Audio';
      case TrackType.text:
        return 'Text';
      case TrackType.effect:
        return 'FX';
    }
  }
}

/// White trim handle shown on the left/right of a selected clip.
class _TrimHandle extends StatelessWidget {
  const _TrimHandle({required this.left, required this.color});

  final bool left;
  final Color color;

  @override
  Widget build(BuildContext context) {
    const radius = Radius.circular(AppTheme.radiusSmall);
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.92),
        borderRadius: BorderRadius.only(
          topLeft: left ? radius : Radius.zero,
          bottomLeft: left ? radius : Radius.zero,
          topRight: left ? Radius.zero : radius,
          bottomRight: left ? Radius.zero : radius,
        ),
      ),
      child: Center(
        child: Container(
          width: 1,
          height: 14,
          color: color.withValues(alpha: 0.6),
        ),
      ),
    );
  }
}

/// Playhead indicator — a red vertical line with a triangle handle that
/// scrolls horizontally with the timeline content.
class _PlayheadIndicator extends StatelessWidget {
  const _PlayheadIndicator({
    required this.currentTimeMs,
    required this.pixelsPerMs,
    required this.scrollController,
  });

  final int currentTimeMs;
  final double pixelsPerMs;
  final ScrollController scrollController;

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: scrollController,
      builder: (context, _) {
        final offset =
            scrollController.hasClients ? scrollController.offset : 0.0;
        final x = currentTimeMs * pixelsPerMs - offset;
        return Stack(
          clipBehavior: Clip.none,
          children: [
            Positioned(
              left: x - AppTheme.playheadWidth / 2,
              top: 0,
              bottom: 0,
              child: Container(
                width: AppTheme.playheadWidth,
                decoration: BoxDecoration(
                  color: AppTheme.playheadColor,
                  boxShadow: [
                    BoxShadow(
                      color: AppTheme.playheadColor.withValues(alpha: 0.5),
                      blurRadius: 4,
                    ),
                  ],
                ),
              ),
            ),
            Positioned(
              left: x - 7,
              top: 0,
              child: const Icon(
                Icons.arrow_drop_down,
                color: AppTheme.playheadColor,
                size: 14,
              ),
            ),
          ],
        );
      },
    );
  }
}

/// Track header shown in the fixed left column. Renders the type icon,
/// name, and per-track toggles (visibility, lock, and mute/solo for audio).
class _TrackHeaderItem extends StatelessWidget {
  const _TrackHeaderItem({
    required this.track,
    required this.trackIndex,
    required this.color,
    required this.isSoloed,
    required this.onSelect,
    required this.onToggleVisibility,
    required this.onToggleLock,
    required this.onToggleMute,
    required this.onToggleSolo,
  });

  final TrackModel track;
  final int trackIndex;
  final Color color;
  final bool isSoloed;
  final VoidCallback onSelect;
  final VoidCallback onToggleVisibility;
  final VoidCallback onToggleLock;
  final VoidCallback onToggleMute;
  final VoidCallback onToggleSolo;

  @override
  Widget build(BuildContext context) {
    final isAudio = track.trackType == TrackType.audio;
    final muted = track.volume <= 0;

    return Container(
      height: AppTheme.trackHeight,
      decoration: BoxDecoration(
        color: trackIndex.isEven ? AppTheme.surface : AppTheme.surfaceVariant,
        border: const Border(
          right: BorderSide(color: AppTheme.border),
          bottom: BorderSide(color: AppTheme.border),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Full-height color stripe.
          Container(width: 4, color: color),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Top: icon + name (tap to select the track).
                Expanded(
                  child: GestureDetector(
                    onTap: onSelect,
                    behavior: HitTestBehavior.opaque,
                    child: Padding(
                      padding: const EdgeInsets.only(left: 6, top: 4),
                      child: Row(
                        children: [
                          AppIcon(
                            _trackIconPath(track.trackType),
                            size: 16,
                            color: color,
                          ),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              track.name,
                              style: context.textTheme.labelMedium?.copyWith(
                                color: track.visible
                                    ? AppTheme.textPrimary
                                    : AppTheme.textDisabled,
                              ),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
                // Bottom: per-track toggles.
                Padding(
                  padding: const EdgeInsets.only(left: 6, bottom: 4),
                  child: Row(
                    children: [
                      _TimelineIconButton(
                        icon: track.visible
                            ? AppIcons.visible
                            : AppIcons.hidden,
                        size: 12,
                        color: track.visible
                            ? AppTheme.textSecondary
                            : AppTheme.textDisabled,
                        onTap: onToggleVisibility,
                      ),
                      const SizedBox(width: 2),
                      _TimelineIconButton(
                        icon: track.locked ? AppIcons.lock : AppIcons.unlock,
                        size: 12,
                        color: track.locked
                            ? AppTheme.warning
                            : AppTheme.textSecondary,
                        onTap: onToggleLock,
                      ),
                      if (isAudio) ...[
                        const SizedBox(width: 2),
                        _LabelToggleButton(
                          label: 'M',
                          active: muted,
                          activeColor: AppTheme.error,
                          onTap: onToggleMute,
                        ),
                        const SizedBox(width: 2),
                        _LabelToggleButton(
                          label: 'S',
                          active: isSoloed,
                          activeColor: AppTheme.warning,
                          onTap: onToggleSolo,
                        ),
                      ],
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 4),
        ],
      ),
    );
  }
}

/// Compact SVG icon button used in the toolbar and track headers.
class _TimelineIconButton extends StatelessWidget {
  const _TimelineIconButton({
    required this.icon,
    this.onTap,
    this.color,
    this.size = 16,
  });

  final String icon;
  final VoidCallback? onTap;
  final Color? color;
  final double size;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = color ?? AppTheme.textSecondary;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: SizedBox(
          width: size + 8,
          height: size + 8,
          child: Center(
            child: SvgPicture.asset(
              icon,
              width: size,
              height: size,
              colorFilter: ColorFilter.mode(
                effectiveColor,
                BlendMode.srcIn,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Small monospace label toggle (M / S) for audio tracks.
class _LabelToggleButton extends StatelessWidget {
  const _LabelToggleButton({
    required this.label,
    required this.active,
    required this.activeColor,
    this.onTap,
  });

  final String label;
  final bool active;
  final Color activeColor;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = active ? activeColor : AppTheme.textSecondary;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          width: 20,
          height: 20,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: active
                ? activeColor.withValues(alpha: 0.18)
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            label,
            style: TextStyle(
              color: effectiveColor,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              fontFamily: 'monospace',
            ),
          ),
        ),
      ),
    );
  }
}

/// Simple bounded LRU cache used for waveform peaks.
class _LruCache<K, V> {
  _LruCache({required this.maxSize});

  final int maxSize;
  final LinkedHashMap<K, V> _map = LinkedHashMap<K, V>();

  V? get(K key) {
    final value = _map.remove(key);
    if (value != null) {
      _map[key] = value;
    }
    return value;
  }

  void put(K key, V value) {
    _map.remove(key);
    _map[key] = value;
    while (_map.length > maxSize) {
      _map.remove(_map.keys.first);
    }
  }

  void clear() => _map.clear();
}
