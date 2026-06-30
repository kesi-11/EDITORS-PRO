import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../../../core/theme/app_theme.dart';

/// Types of safe zone overlays that can be displayed on the preview viewport.
///
/// Broadcast standards define safe areas to ensure important content is
/// visible on all displays. The action safe area (90%) guarantees that
/// action within it is visible on most CRT/overscan displays. The title
/// safe area (80%) guarantees text is readable without distortion.
enum SafeZoneType {
  /// 90% of frame — content within this area is visible on virtually all displays.
  actionSafe,

  /// 80% of frame — text/titles within this area are readable on all displays.
  titleSafe,

  /// Thin crosshair through the exact center of the frame.
  centerCross,

  /// Rule of thirds grid — 2 vertical + 2 horizontal lines dividing the frame
  /// into 9 equal regions for composition guidance.
  thirds,

  /// Small circle + crosshair at the exact center of the frame.
  centerMarker,
}

/// A semi-transparent overlay widget that renders safe-zone guides on top
/// of the video preview viewport.
///
/// Used by video editors to ensure titles and key action fall within
/// broadcast-safe areas. Multiple zone types can be enabled simultaneously.
///
/// Usage:
/// ```dart
/// Stack(
///   children: [
///     VideoPreview(),
///     SafeZonesOverlay(
///       enabledZones: {SafeZoneType.actionSafe, SafeZoneType.titleSafe},
///     ),
///   ],
/// )
/// ```
class SafeZonesOverlay extends StatelessWidget {
  /// Which safe zone types to display. Defaults to action + title safe.
  final Set<SafeZoneType> enabledZones;

  const SafeZonesOverlay({
    super.key,
    this.enabledZones = const {
      SafeZoneType.actionSafe,
      SafeZoneType.titleSafe,
    },
  });

  @override
  Widget build(BuildContext context) {
    if (enabledZones.isEmpty) return const SizedBox.shrink();

    return CustomPaint(
      painter: SafeZonesPainter(enabledZones: enabledZones),
      size: Size.infinite,
    );
  }
}

/// Custom painter that renders all enabled safe zone types.
///
/// Uses dashed lines for the safe rectangles and solid thin lines for
/// grid/crosshair overlays. Labels are drawn at the top-left corner of
/// each safe rectangle.
class SafeZonesPainter extends CustomPainter {
  final Set<SafeZoneType> enabledZones;

  SafeZonesPainter({required this.enabledZones});

  // ─── Color constants ──────────────────────────────────────────
  static const Color _lineColor = Color(0xCCFFFFFF); // 80% white
  static const Color _labelColor = Color(0xBBFFFFFF); // 73% white
  static const Color _centerColor = Color(0xAAFFFFFF); // 67% white
  static const double _dashLength = 8.0;
  static const double _dashGap = 5.0;
  static const double _lineWidth = 1.0;
  static const double _labelFontSize = 9.0;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.isEmpty) return;

    // Center crosshair (drawn first so it's behind everything else)
    if (enabledZones.contains(SafeZoneType.centerCross)) {
      _drawCenterCross(canvas, size);
    }

    // Rule of thirds grid
    if (enabledZones.contains(SafeZoneType.thirds)) {
      _drawThirdsGrid(canvas, size);
    }

    // Action safe rectangle (90%)
    if (enabledZones.contains(SafeZoneType.actionSafe)) {
      _drawSafeRect(
        canvas,
        size,
        fraction: 0.90,
        label: 'ACTION SAFE',
        color: _lineColor.withOpacity(0.6),
      );
    }

    // Title safe rectangle (80%)
    if (enabledZones.contains(SafeZoneType.titleSafe)) {
      _drawSafeRect(
        canvas,
        size,
        fraction: 0.80,
        label: 'TITLE SAFE',
        color: _lineColor.withOpacity(0.8),
      );
    }

    // Center marker (drawn last so it's on top)
    if (enabledZones.contains(SafeZoneType.centerMarker)) {
      _drawCenterMarker(canvas, size);
    }
  }

  /// Draw a dashed safe-zone rectangle at the given [fraction] of the
  /// frame size, with a [label] in the top-left corner.
  void _drawSafeRect(
    Canvas canvas,
    Size size, {
    required double fraction,
    required String label,
    required Color color,
  }) {
    final insetX = size.width * (1 - fraction) / 2;
    final insetY = size.height * (1 - fraction) / 2;
    final rect = Rect.fromLTWH(
      insetX,
      insetY,
      size.width * fraction,
      size.height * fraction,
    );

    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = _lineWidth;

    _drawDashedRect(canvas, rect, paint);

    // Draw label at top-left corner of the safe rect
    _drawLabel(canvas, label, Offset(rect.left + 4, rect.top + 2));
  }

  /// Draw a dashed rectangle by breaking each edge into dash/gap segments.
  void _drawDashedRect(Canvas canvas, Rect rect, Paint paint) {
    // Top edge
    _drawDashedLine(canvas, rect.topLeft, rect.topRight, paint);
    // Right edge
    _drawDashedLine(canvas, rect.topRight, rect.bottomRight, paint);
    // Bottom edge
    _drawDashedLine(canvas, rect.bottomRight, rect.bottomLeft, paint);
    // Left edge
    _drawDashedLine(canvas, rect.bottomLeft, rect.topLeft, paint);
  }

  /// Draw a dashed line from [start] to [end] using [_dashLength] dashes
  /// separated by [_dashGap] gaps.
  void _drawDashedLine(
    Canvas canvas,
    Offset start,
    Offset end,
    Paint paint,
  ) {
    final dx = end.dx - start.dx;
    final dy = end.dy - start.dy;
    final totalLength = math.sqrt(dx * dx + dy * dy);
    if (totalLength == 0) return;

    final unitX = dx / totalLength;
    final unitY = dy / totalLength;

    double drawn = 0;
    bool drawing = true;

    while (drawn < totalLength) {
      final segmentLength = drawing ? _dashLength : _dashGap;
      final remaining = totalLength - drawn;
      final currentLength = segmentLength.clamp(0.0, remaining);

      if (drawing) {
        final segStart = Offset(
          start.dx + unitX * drawn,
          start.dy + unitY * drawn,
        );
        final segEnd = Offset(
          start.dx + unitX * (drawn + currentLength),
          start.dy + unitY * (drawn + currentLength),
        );
        canvas.drawLine(segStart, segEnd, paint);
      }

      drawn += currentLength;
      drawing = !drawing;
    }
  }

  /// Draw a small text label at [position].
  void _drawLabel(Canvas canvas, String text, Offset position) {
    final textSpan = TextSpan(
      text: text,
      style: const TextStyle(
        color: _labelColor,
        fontSize: _labelFontSize,
        fontWeight: FontWeight.w600,
        fontFamily: 'Inter',
        letterSpacing: 0.5,
      ),
    );
    final textPainter = TextPainter(
      text: textSpan,
      textDirection: TextDirection.ltr,
    );
    textPainter.layout();
    textPainter.paint(canvas, position);
  }

  /// Draw a thin crosshair through the center of the frame.
  void _drawCenterCross(Canvas canvas, Size size) {
    final cx = size.width / 2;
    final cy = size.height / 2;
    final paint = Paint()
      ..color = _centerColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 0.5;

    // Horizontal line
    canvas.drawLine(Offset(0, cy), Offset(size.width, cy), paint);
    // Vertical line
    canvas.drawLine(Offset(cx, 0), Offset(cx, size.height), paint);
  }

  /// Draw the rule of thirds grid — 2 vertical + 2 horizontal lines
  /// dividing the frame into 9 equal regions.
  void _drawThirdsGrid(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = _lineColor.withOpacity(0.35)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 0.5;

    // Vertical third lines
    for (int i = 1; i <= 2; i++) {
      final x = size.width * i / 3;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }

    // Horizontal third lines
    for (int i = 1; i <= 2; i++) {
      final y = size.height * i / 3;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  /// Draw a center marker — a small circle plus a short crosshair at the
  /// exact center of the frame.
  void _drawCenterMarker(Canvas canvas, Size size) {
    final cx = size.width / 2;
    final cy = size.height / 2;
    final center = Offset(cx, cy);

    // Small circle
    final circlePaint = Paint()
      ..color = _centerColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.0;
    canvas.drawCircle(center, 6.0, circlePaint);

    // Short crosshair arms (each 14px from center, with a 6px gap for the circle)
    final armPaint = Paint()
      ..color = _centerColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 0.8;

    const armLength = 14.0;
    const gap = 7.0; // just outside the circle radius

    // Right arm
    canvas.drawLine(
      Offset(cx + gap, cy),
      Offset(cx + armLength, cy),
      armPaint,
    );
    // Left arm
    canvas.drawLine(
      Offset(cx - gap, cy),
      Offset(cx - armLength, cy),
      armPaint,
    );
    // Bottom arm
    canvas.drawLine(
      Offset(cx, cy + gap),
      Offset(cx, cy + armLength),
      armPaint,
    );
    // Top arm
    canvas.drawLine(
      Offset(cx, cy - gap),
      Offset(cx, cy - armLength),
      armPaint,
    );
  }

  @override
  bool shouldRepaint(covariant SafeZonesPainter oldDelegate) =>
      enabledZones != oldDelegate.enabledZones;
}
