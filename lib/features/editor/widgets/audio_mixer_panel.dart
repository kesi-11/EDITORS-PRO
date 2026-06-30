import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';

/// Professional multi-track audio mixer panel inspired by DaVinci Resolve
/// and Adobe Premiere Pro's audio mixer consoles.
///
/// Features per-track volume faders, pan knobs, mute/solo, VU meters,
/// automation mode selectors, and a master strip with stereo metering.
class AudioMixerPanel extends ConsumerStatefulWidget {
  const AudioMixerPanel({super.key});

  @override
  ConsumerState<AudioMixerPanel> createState() => _AudioMixerPanelState();
}

class _AudioMixerPanelState extends ConsumerState<AudioMixerPanel> {
  // ─── Per-track state ──────────────────────────────────────────────
  final Map<String, double> _panPositions = {};
  final Map<String, String> _automationModes = {};
  final Map<String, double> _vuLevels = {};
  final Map<String, double> _vuPeakHold = {};

  // ─── Master state ─────────────────────────────────────────────────
  double _masterVuLeft = 0.0;
  double _masterVuRight = 0.0;
  double _masterPeakLeft = 0.0;
  double _masterPeakRight = 0.0;
  bool _masterMuted = false;
  double _preMuteMasterVolume = 1.0;

  // ─── Solo state ───────────────────────────────────────────────────
  String? _soloTrackId;

  // ─── VU simulation timer ──────────────────────────────────────────
  Timer? _vuTimer;
  final _random = math.Random();

  static const List<String> _automationModeNames = [
    'Off',
    'Read',
    'Write',
    'Touch',
    'Latch',
  ];

  @override
  void initState() {
    super.initState();
    _vuTimer = Timer.periodic(const Duration(milliseconds: 50), (_) {
      if (!mounted) return;
      _updateVuLevels();
    });
  }

  @override
  void dispose() {
    _vuTimer?.cancel();
    super.dispose();
  }

  /// Simulate VU meter levels based on track volumes.
  /// In production this would poll the engine's real audio levels.
  void _updateVuLevels() {
    final project = ref.read(currentProjectProvider);
    if (project == null) return;

    final editorState = ref.read(editorProvider);
    final isPlaying = editorState.isPlaying;

    for (final track in project.tracks) {
      final baseVolume = track.volume;
      final isMuted = !track.visible;
      final isSoloed = _soloTrackId != null && _soloTrackId != track.id;

      double level;
      if (!isPlaying || isMuted || isSoloed) {
        level = _vuLevels[track.id] ?? 0.0;
        // Decay to zero when not playing
        level = math.max(0.0, level - 0.08);
      } else {
        // Simulate: random flutter scaled by volume
        final flutter = 0.3 + _random.nextDouble() * 0.7;
        final target = baseVolume * flutter;
        // Smooth towards target
        final current = _vuLevels[track.id] ?? 0.0;
        level = current + (target - current) * 0.4;
      }

      level = level.clamp(0.0, 1.2);

      // Peak hold
      final currentPeak = _vuPeakHold[track.id] ?? 0.0;
      if (level > currentPeak) {
        _vuPeakHold[track.id] = level;
      } else {
        _vuPeakHold[track.id] = math.max(0.0, currentPeak - 0.005);
      }

      _vuLevels[track.id] = level;
    }

    // Master VU: combine all track levels
    if (!isPlaying || _masterMuted) {
      _masterVuLeft = math.max(0.0, _masterVuLeft - 0.08);
      _masterVuRight = math.max(0.0, _masterVuRight - 0.08);
    } else {
      double combinedLeft = 0.0;
      double combinedRight = 0.0;
      for (final track in project.tracks) {
        final isMuted = !track.visible;
        final isSoloed = _soloTrackId != null && _soloTrackId != track.id;
        if (isMuted || isSoloed) continue;

        final trackLevel = _vuLevels[track.id] ?? 0.0;
        final pan = _panPositions[track.id] ?? 0.0;

        // Pan law: -3dB center, linear pan
        final leftGain = math.cos((pan + 1.0) * math.pi / 4.0);
        final rightGain = math.sin((pan + 1.0) * math.pi / 4.0);

        combinedLeft += trackLevel * leftGain;
        combinedRight += trackLevel * rightGain;
      }

      // Soft clamp combined levels
      final masterVol = editorState.masterVolume;
      final targetL = (combinedLeft * masterVol).clamp(0.0, 1.2);
      final targetR = (combinedRight * masterVol).clamp(0.0, 1.2);

      _masterVuLeft += (targetL - _masterVuLeft) * 0.4;
      _masterVuRight += (targetR - _masterVuRight) * 0.4;
    }

    // Master peak hold
    if (_masterVuLeft > _masterPeakLeft) {
      _masterPeakLeft = _masterVuLeft;
    } else {
      _masterPeakLeft = math.max(0.0, _masterPeakLeft - 0.005);
    }
    if (_masterVuRight > _masterPeakRight) {
      _masterPeakRight = _masterVuRight;
    } else {
      _masterPeakRight = math.max(0.0, _masterPeakRight - 0.005);
    }

    setState(() {});
  }

  // ─── Build ────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final project = ref.watch(currentProjectProvider);
    if (project == null) return const SizedBox.shrink();

    return Container(
      color: AppTheme.background,
      child: Column(
        children: [
          _buildMixerHeader(),
          Expanded(child: _buildMixerStrips(project)),
        ],
      ),
    );
  }

  // ─── Header ───────────────────────────────────────────────────────

  Widget _buildMixerHeader() {
    return Container(
      height: 36,
      decoration: const BoxDecoration(
        color: AppTheme.surface,
        border: Border(
          bottom: BorderSide(color: AppTheme.border, width: 1),
        ),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          const Icon(Icons.graphic_eq, size: 16, color: AppTheme.secondary),
          const SizedBox(width: 8),
          Text(
            'Audio Mixer',
            style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: AppTheme.textPrimary,
                  fontWeight: FontWeight.w600,
                ),
          ),
          const Spacer(),
          // Reset all button
          _headerButton('Reset', () {
            setState(() {
              _soloTrackId = null;
              _masterMuted = false;
              for (final key in _panPositions.keys.toList()) {
                _panPositions[key] = 0.0;
              }
              for (final key in _automationModes.keys.toList()) {
                _automationModes[key] = 'Off';
              }
            });
          }),
        ],
      ),
    );
  }

  Widget _headerButton(String label, VoidCallback onTap) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: AppTheme.surfaceVariant,
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          border: Border.all(color: AppTheme.border),
        ),
        child: Text(
          label,
          style: const TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 11,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }

  // ─── Mixer Strips ─────────────────────────────────────────────────

  Widget _buildMixerStrips(ProjectModel project) {
    final tracks = project.tracks;

    return Row(
      children: [
        // Scrollable track strips
        Expanded(
          child: ListView.separated(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
            itemCount: tracks.length,
            separatorBuilder: (_, __) => const SizedBox(width: 2),
            itemBuilder: (context, index) {
              return _buildTrackStrip(tracks[index]);
            },
          ),
        ),
        // Divider
        Container(width: 1, color: AppTheme.borderLight),
        // Master strip
        _buildMasterStrip(),
      ],
    );
  }

  // ─── Track Strip ──────────────────────────────────────────────────

  Widget _buildTrackStrip(TrackModel track) {
    final isMuted = !track.visible;
    final isSoloed = _soloTrackId == track.id;
    final hasSoloActive = _soloTrackId != null;
    final isDimmed = hasSoloActive && !isSoloed;
    final volume = track.volume;
    final pan = _panPositions[track.id] ?? 0.0;
    final autoMode = _automationModes[track.id] ?? 'Off';
    final vuLevel = _vuLevels[track.id] ?? 0.0;
    final vuPeak = _vuPeakHold[track.id] ?? 0.0;

    final trackColor = _trackColor(track.trackType);

    return Opacity(
      opacity: isDimmed ? 0.4 : 1.0,
      child: Container(
        width: 60,
        decoration: BoxDecoration(
          color: AppTheme.surface,
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          border: Border.all(
            color: isMuted ? AppTheme.error.withOpacity(0.3) : AppTheme.border,
            width: 1,
          ),
        ),
        child: Column(
          children: [
            // Track name
            _buildTrackName(track, trackColor),
            const SizedBox(height: 4),

            // Automation mode
            _buildAutomationSelector(track.id, autoMode),
            const SizedBox(height: 4),

            // Pan knob
            _buildPanKnob(track.id, pan),
            const SizedBox(height: 4),

            // Volume fader + VU meter
            Expanded(
              child: _buildFaderAndMeter(
                trackId: track.id,
                volume: volume,
                vuLevel: vuLevel,
                vuPeak: vuPeak,
                trackColor: trackColor,
                isMuted: isMuted,
              ),
            ),
            const SizedBox(height: 4),

            // Volume label
            Text(
              '${(volume * 100).round()}%',
              style: TextStyle(
                color: isMuted ? AppTheme.error : AppTheme.textSecondary,
                fontSize: 9,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 4),

            // Mute / Solo buttons
            _buildMuteSoloButtons(track, isMuted, isSoloed),
            const SizedBox(height: 6),
          ],
        ),
      ),
    );
  }

  Widget _buildTrackName(TrackModel track, Color trackColor) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 4),
      decoration: BoxDecoration(
        color: trackColor.withOpacity(0.15),
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(AppTheme.radiusSmall),
          topRight: Radius.circular(AppTheme.radiusSmall),
        ),
      ),
      child: Text(
        track.name.length > 7 ? '${track.name.substring(0, 6)}…' : track.name,
        textAlign: TextAlign.center,
        style: TextStyle(
          color: trackColor,
          fontSize: 9,
          fontWeight: FontWeight.w700,
        ),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }

  Widget _buildAutomationSelector(String trackId, String currentMode) {
    return GestureDetector(
      onTap: () {
        final currentIndex = _automationModeNames.indexOf(currentMode);
        final nextIndex = (currentIndex + 1) % _automationModeNames.length;
        setState(() {
          _automationModes[trackId] = _automationModeNames[nextIndex];
        });
      },
      child: Container(
        width: 46,
        height: 16,
        decoration: BoxDecoration(
          color: _autoModeColor(currentMode).withOpacity(0.15),
          borderRadius: BorderRadius.circular(3),
          border: Border.all(
            color: _autoModeColor(currentMode).withOpacity(0.4),
            width: 0.5,
          ),
        ),
        alignment: Alignment.center,
        child: Text(
          currentMode,
          style: TextStyle(
            color: _autoModeColor(currentMode),
            fontSize: 8,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.5,
          ),
        ),
      ),
    );
  }

  Color _autoModeColor(String mode) {
    switch (mode) {
      case 'Off':
        return AppTheme.textDisabled;
      case 'Read':
        return AppTheme.info;
      case 'Write':
        return AppTheme.error;
      case 'Touch':
        return AppTheme.warning;
      case 'Latch':
        return AppTheme.secondary;
      default:
        return AppTheme.textDisabled;
    }
  }

  // ─── Fader + VU Meter ─────────────────────────────────────────────

  Widget _buildFaderAndMeter({
    required String trackId,
    required double volume,
    required double vuLevel,
    required double vuPeak,
    required Color trackColor,
    required bool isMuted,
  }) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // VU meter
        _buildVuMeter(
          level: isMuted ? 0.0 : vuLevel,
          peakHold: isMuted ? 0.0 : vuPeak,
        ),
        const SizedBox(width: 3),
        // Vertical fader
        _buildVerticalFader(
          trackId: trackId,
          volume: volume,
          trackColor: trackColor,
        ),
      ],
    );
  }

  Widget _buildVerticalFader({
    required String trackId,
    required double volume,
    required Color trackColor,
  }) {
    // Fader range: 0.0 to 2.0 (0% to 200%)
    final faderValue = volume.clamp(0.0, 2.0);

    return SizedBox(
      width: 22,
      child: _FaderTrack(
        value: faderValue,
        minValue: 0.0,
        maxValue: 2.0,
        trackColor: trackColor,
        onChanged: (newVolume) {
          ref.read(editorProvider.notifier).setTrackVolume(trackId, newVolume);
        },
      ),
    );
  }

  // ─── VU Meter ─────────────────────────────────────────────────────

  Widget _buildVuMeter({
    required double level,
    required double peakHold,
    bool isStereo = false,
    double? rightLevel,
    double? rightPeakHold,
  }) {
    if (isStereo) {
      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _vuBar(level, peakHold, width: 5),
          const SizedBox(width: 1),
          _vuBar(rightLevel ?? 0.0, rightPeakHold ?? 0.0, width: 5),
        ],
      );
    }
    return _vuBar(level, peakHold, width: 7);
  }

  Widget _vuBar(double level, double peakHold, {double width = 7}) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(2),
      child: CustomPaint(
        size: Size(width, double.infinity),
        painter: _VuMeterPainter(
          level: level.clamp(0.0, 1.2),
          peakHold: peakHold.clamp(0.0, 1.2),
        ),
      ),
    );
  }

  // ─── Pan Knob ─────────────────────────────────────────────────────

  Widget _buildPanKnob(String trackId, double pan) {
    return SizedBox(
      width: 30,
      height: 30,
      child: GestureDetector(
        onPanUpdate: (details) {
          setState(() {
            // Horizontal drag maps to pan -100 to +100
            final delta = details.delta.dx;
            final current = _panPositions[trackId] ?? 0.0;
            _panPositions[trackId] = (current + delta * 0.02).clamp(-1.0, 1.0);
          });
        },
        onDoubleTap: () {
          setState(() {
            _panPositions[trackId] = 0.0;
          });
        },
        child: CustomPaint(
          painter: _PanKnobPainter(
            panValue: pan,
            knobColor: AppTheme.surfaceVariant,
            indicatorColor: AppTheme.secondary,
          ),
        ),
      ),
    );
  }

  // ─── Mute / Solo Buttons ──────────────────────────────────────────

  Widget _buildMuteSoloButtons(TrackModel track, bool isMuted, bool isSoloed) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Mute
        GestureDetector(
          onTap: () {
            ref.read(editorProvider.notifier).toggleTrackVisibility(track.id);
          },
          child: Container(
            width: 22,
            height: 18,
            decoration: BoxDecoration(
              color: isMuted ? AppTheme.error : AppTheme.surfaceVariant,
              borderRadius: BorderRadius.circular(3),
              border: Border.all(
                color: isMuted ? AppTheme.error : AppTheme.border,
                width: 0.5,
              ),
            ),
            alignment: Alignment.center,
            child: Text(
              'M',
              style: TextStyle(
                color: isMuted ? Colors.white : AppTheme.textSecondary,
                fontSize: 8,
                fontWeight: FontWeight.w800,
              ),
            ),
          ),
        ),
        const SizedBox(width: 3),
        // Solo
        GestureDetector(
          onTap: () {
            setState(() {
              if (_soloTrackId == track.id) {
                _soloTrackId = null;
              } else {
                _soloTrackId = track.id;
              }
            });
          },
          child: Container(
            width: 22,
            height: 18,
            decoration: BoxDecoration(
              color: isSoloed ? AppTheme.warning : AppTheme.surfaceVariant,
              borderRadius: BorderRadius.circular(3),
              border: Border.all(
                color: isSoloed ? AppTheme.warning : AppTheme.border,
                width: 0.5,
              ),
            ),
            alignment: Alignment.center,
            child: Text(
              'S',
              style: TextStyle(
                color: isSoloed ? Colors.black : AppTheme.textSecondary,
                fontSize: 8,
                fontWeight: FontWeight.w800,
              ),
            ),
          ),
        ),
      ],
    );
  }

  // ─── Master Strip ─────────────────────────────────────────────────

  Widget _buildMasterStrip() {
    final editorState = ref.watch(editorProvider);
    final masterVolume = editorState.masterVolume;

    return Container(
      width: 72,
      decoration: BoxDecoration(
        color: AppTheme.surfaceVariant,
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
        border: Border.all(color: AppTheme.borderLight),
      ),
      child: Column(
        children: [
          // Master label
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 5),
            decoration: const BoxDecoration(
              color: AppTheme.primaryDark,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(AppTheme.radiusSmall),
                topRight: Radius.circular(AppTheme.radiusSmall),
              ),
            ),
            child: const Text(
              'MASTER',
              textAlign: TextAlign.center,
              style: TextStyle(
                color: Colors.white,
                fontSize: 9,
                fontWeight: FontWeight.w800,
                letterSpacing: 1.0,
              ),
            ),
          ),
          const SizedBox(height: 6),

          // Stereo VU meter + fader
          Expanded(
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                // Stereo VU meter
                _buildVuMeter(
                  level: _masterMuted ? 0.0 : _masterVuLeft,
                  peakHold: _masterMuted ? 0.0 : _masterPeakLeft,
                  isStereo: true,
                  rightLevel: _masterMuted ? 0.0 : _masterVuRight,
                  rightPeakHold: _masterMuted ? 0.0 : _masterPeakRight,
                ),
                const SizedBox(width: 4),
                // Master fader
                SizedBox(
                  width: 26,
                  child: _FaderTrack(
                    value: masterVolume,
                    minValue: 0.0,
                    maxValue: 1.0,
                    trackColor: AppTheme.primary,
                    showDbScale: true,
                    onChanged: (newVolume) {
                      ref.read(editorProvider.notifier).setMasterVolume(newVolume);
                    },
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 4),

          // Master volume label
          Text(
            '${(masterVolume * 100).round()}%',
            style: TextStyle(
              color: _masterMuted ? AppTheme.error : AppTheme.primaryLight,
              fontSize: 10,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 4),

          // Master mute
          GestureDetector(
            onTap: () {
              setState(() {
                if (!_masterMuted) {
                  // Muting: save current volume before setting to 0
                  _preMuteMasterVolume = masterVolume > 0 ? masterVolume : 1.0;
                  _masterMuted = true;
                  ref.read(editorProvider.notifier).setMasterVolume(0.0);
                } else {
                  // Unmuting: restore saved volume
                  _masterMuted = false;
                  ref.read(editorProvider.notifier).setMasterVolume(_preMuteMasterVolume);
                }
              });
            },
            child: Container(
              width: 36,
              height: 22,
              decoration: BoxDecoration(
                color: _masterMuted ? AppTheme.error : AppTheme.primary.withOpacity(0.2),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: _masterMuted ? AppTheme.error : AppTheme.primary,
                  width: 0.5,
                ),
              ),
              alignment: Alignment.center,
              child: Text(
                'M',
                style: TextStyle(
                  color: _masterMuted ? Colors.white : AppTheme.primaryLight,
                  fontSize: 10,
                  fontWeight: FontWeight.w800,
                ),
              ),
            ),
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }

  // ─── Helpers ──────────────────────────────────────────────────────

  Color _trackColor(TrackType type) {
    switch (type) {
      case TrackType.audio:
        return AppTheme.audioTrackColor;
      case TrackType.video:
        return AppTheme.videoTrackColor;
      case TrackType.text:
        return AppTheme.textTrackColor;
      case TrackType.effect:
        return AppTheme.effectTrackColor;
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Custom Painters
// ═══════════════════════════════════════════════════════════════════════

/// VU meter painter with green/yellow/red zones and peak hold indicator.
class _VuMeterPainter extends CustomPainter {
  final double level;
  final double peakHold;

  _VuMeterPainter({
    required this.level,
    required this.peakHold,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final width = size.width;
    final height = size.height;

    // Background
    final bgPaint = Paint()..color = AppTheme.background;
    canvas.drawRect(Rect.fromLTWH(0, 0, width, height), bgPaint);

    // Segment dimensions
    const segmentGap = 1.0;
    const segmentHeight = 2.0;
    final totalSegments = (height / (segmentHeight + segmentGap)).floor();

    // Draw segments bottom-up
    for (int i = 0; i < totalSegments; i++) {
      final y = height - (i + 1) * (segmentHeight + segmentGap);
      final segmentLevel = i / totalSegments;

      Color segmentColor;
      if (segmentLevel < 0.6) {
        // Green zone (0-60%)
        segmentColor = const Color(0xFF00E676);
      } else if (segmentLevel < 0.85) {
        // Yellow zone (60-85%)
        segmentColor = const Color(0xFFFFD600);
      } else {
        // Red zone (85-100%)
        segmentColor = const Color(0xFFFF1744);
      }

      // Determine if this segment is lit
      final normalizedLevel = (level / 1.2).clamp(0.0, 1.0);
      final isLit = segmentLevel <= normalizedLevel;

      final paint = Paint()
        ..color = isLit
            ? segmentColor
            : segmentColor.withOpacity(0.12);

      canvas.drawRect(
        Rect.fromLTWH(0, y, width, segmentHeight),
        paint,
      );
    }

    // Peak hold indicator
    if (peakHold > 0.01) {
      final normalizedPeak = (peakHold / 1.2).clamp(0.0, 1.0);
      final peakY = height - normalizedPeak * height;
      Color peakColor;
      final peakSegmentLevel = normalizedPeak;
      if (peakSegmentLevel < 0.6) {
        peakColor = const Color(0xFF00E676);
      } else if (peakSegmentLevel < 0.85) {
        peakColor = const Color(0xFFFFD600);
      } else {
        peakColor = const Color(0xFFFF1744);
      }

      final peakPaint = Paint()
        ..color = peakColor
        ..strokeWidth = 1.5;
      canvas.drawLine(
        Offset(0, peakY),
        Offset(width, peakY),
        peakPaint,
      );
    }
  }

  @override
  bool shouldRepaint(_VuMeterPainter oldDelegate) {
    return oldDelegate.level != level || oldDelegate.peakHold != peakHold;
  }
}

/// Pan knob painter — circular knob with position indicator.
class _PanKnobPainter extends CustomPainter {
  final double panValue; // -1.0 to +1.0
  final Color knobColor;
  final Color indicatorColor;

  _PanKnobPainter({
    required this.panValue,
    required this.knobColor,
    required this.indicatorColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.width / 2 - 2;

    // Knob body
    final knobPaint = Paint()
      ..color = knobColor
      ..style = PaintingStyle.fill;
    canvas.drawCircle(center, radius, knobPaint);

    // Outer ring
    final ringPaint = Paint()
      ..color = AppTheme.borderLight
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.0;
    canvas.drawCircle(center, radius, ringPaint);

    // Tick marks around the knob (9 o'clock to 3 o'clock arc)
    final tickPaint = Paint()
      ..color = AppTheme.textDisabled.withOpacity(0.5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 0.5;

    for (int i = 0; i <= 10; i++) {
      final angle = -math.pi * 0.75 + (i / 10) * math.pi * 1.5;
      final innerR = radius - 3;
      final outerR = radius - 1;
      canvas.drawLine(
        Offset(
          center.dx + innerR * math.cos(angle),
          center.dy + innerR * math.sin(angle),
        ),
        Offset(
          center.dx + outerR * math.cos(angle),
          center.dy + outerR * math.sin(angle),
        ),
        tickPaint,
      );
    }

    // Indicator line
    // Pan -1.0 = 9 o'clock (-135°), 0.0 = 12 o'clock (-90°), +1.0 = 3 o'clock (-45°)
    final indicatorAngle = -math.pi / 2 + panValue * math.pi * 0.75;
    final indicatorPaint = Paint()
      ..color = indicatorColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0
      ..strokeCap = StrokeCap.round;

    canvas.drawLine(
      center,
      Offset(
        center.dx + (radius - 4) * math.cos(indicatorAngle),
        center.dy + (radius - 4) * math.sin(indicatorAngle),
      ),
      indicatorPaint,
    );

    // Center dot
    final dotPaint = Paint()
      ..color = indicatorColor.withOpacity(0.6)
      ..style = PaintingStyle.fill;
    canvas.drawCircle(center, 2, dotPaint);
  }

  @override
  bool shouldRepaint(_PanKnobPainter oldDelegate) {
    return oldDelegate.panValue != panValue;
  }
}

/// Vertical fader track with draggable thumb.
class _FaderTrack extends StatefulWidget {
  final double value;
  final double minValue;
  final double maxValue;
  final Color trackColor;
  final bool showDbScale;
  final ValueChanged<double> onChanged;

  const _FaderTrack({
    required this.value,
    required this.minValue,
    required this.maxValue,
    required this.trackColor,
    this.showDbScale = false,
    required this.onChanged,
  });

  @override
  State<_FaderTrack> createState() => _FaderTrackState();
}

class _FaderTrackState extends State<_FaderTrack> {
  double _dragStartY = 0;
  double _dragStartValue = 0;

  @override
  Widget build(BuildContext context) {
    final normalizedValue =
        ((widget.value - widget.minValue) / (widget.maxValue - widget.minValue))
            .clamp(0.0, 1.0);

    return LayoutBuilder(
      builder: (context, constraints) {
        final trackHeight = constraints.maxHeight;
        final thumbY = trackHeight * (1.0 - normalizedValue);

        return GestureDetector(
          onVerticalDragStart: (details) {
            _dragStartY = details.localPosition.dy;
            _dragStartValue = widget.value;
          },
          onVerticalDragUpdate: (details) {
            final dy = details.localPosition.dy - _dragStartY;
            final delta = -dy / trackHeight;
            final newValue =
                (_dragStartValue + delta * (widget.maxValue - widget.minValue))
                    .clamp(widget.minValue, widget.maxValue);
            widget.onChanged(newValue);
          },
          onDoubleTap: () {
            // Double-tap resets to unity (100%)
            widget.onChanged(1.0.clamp(widget.minValue, widget.maxValue));
          },
          child: CustomPaint(
            size: Size(constraints.maxWidth, trackHeight),
            painter: _FaderTrackPainter(
              normalizedValue: normalizedValue,
              trackColor: widget.trackColor,
              showDbScale: widget.showDbScale,
              maxValue: widget.maxValue,
              thumbY: thumbY,
            ),
          ),
        );
      },
    );
  }
}

/// Fader track painter — thin colored line with draggable thumb indicator.
class _FaderTrackPainter extends CustomPainter {
  final double normalizedValue;
  final Color trackColor;
  final bool showDbScale;
  final double maxValue;
  final double thumbY;

  _FaderTrackPainter({
    required this.normalizedValue,
    required this.trackColor,
    required this.showDbScale,
    required this.maxValue,
    required this.thumbY,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final centerX = size.width / 2;
    final trackTop = 0.0;
    final trackBottom = size.height;

    // Track background (thin line)
    final bgPaint = Paint()
      ..color = AppTheme.border.withOpacity(0.5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round;
    canvas.drawLine(
      Offset(centerX, trackTop + 8),
      Offset(centerX, trackBottom - 8),
      bgPaint,
    );

    // Active track fill (from bottom to thumb position)
    final activePaint = Paint()
      ..color = trackColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round;
    canvas.drawLine(
      Offset(centerX, trackBottom - 8),
      Offset(centerX, thumbY + 8),
      activePaint,
    );

    // 0 dB reference line (at 50% for 0-200%, at 100% for 0-100%)
    final zeroDbFraction = maxValue == 2.0 ? 0.5 : 1.0;
    final zeroDbY = trackBottom - (trackBottom - 16) * zeroDbFraction;
    final refPaint = Paint()
      ..color = AppTheme.textDisabled.withOpacity(0.3)
      ..strokeWidth = 0.5;
    canvas.drawLine(
      Offset(centerX - 6, zeroDbY),
      Offset(centerX + 6, zeroDbY),
      refPaint,
    );

    // dB scale marks
    if (showDbScale) {
      final textStyle = TextStyle(
        color: AppTheme.textDisabled,
        fontSize: 7,
      );
      // 0 dB
      _drawDbMark(canvas, '0', 1.0, size.height, textStyle);
      // -6 dB
      _drawDbMark(canvas, '-6', 0.5, size.height, textStyle);
      // -∞
      _drawDbMark(canvas, '-∞', 0.0, size.height, textStyle);
    }

    // Thumb indicator
    final thumbRect = RRect.fromRectAndRadius(
      Rect.fromCenter(
        center: Offset(centerX, thumbY + 8),
        width: size.width - 2,
        height: 10,
      ),
      const Radius.circular(2),
    );

    final thumbPaint = Paint()
      ..color = trackColor
      ..style = PaintingStyle.fill;
    canvas.drawRRect(thumbRect, thumbPaint);

    // Thumb grip lines
    final gripPaint = Paint()
      ..color = Colors.white.withOpacity(0.5)
      ..strokeWidth = 0.5;
    for (int i = -1; i <= 1; i++) {
      final gy = thumbY + 8 + i * 2.0;
      canvas.drawLine(
        Offset(centerX - 4, gy),
        Offset(centerX + 4, gy),
        gripPaint,
      );
    }
  }

  void _drawDbMark(Canvas canvas, String label, double fraction,
      double height, TextStyle style) {
    final y = height - 8 - (height - 16) * fraction;
    final textSpan = TextSpan(text: label, style: style);
    final tp = TextPainter(
      text: textSpan,
      textDirection: TextDirection.ltr,
    );
    tp.layout();
    tp.paint(canvas, Offset(1, y - tp.height / 2));
  }

  @override
  bool shouldRepaint(_FaderTrackPainter oldDelegate) {
    return oldDelegate.normalizedValue != normalizedValue ||
        oldDelegate.thumbY != thumbY;
  }
}
