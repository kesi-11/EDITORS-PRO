import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_icons.dart';
import '../../../core/widgets/app_icon.dart';
import '../providers/editor_provider.dart';
import '../../projects/providers/project_provider.dart';

/// Top toolbar for the editor screen — DaVinci Resolve / Premiere Pro style.
///
/// Layout (left → right):
///  1. Back button (navigates to '/')
///  2. Project name + resolution badge (pill, primaryLight)
///  3. Divider
///  4. Undo / 5. Redo / 6. Save (primaryLight)
///  7. Divider
///  8. Skip to start / 9. Play-Pause (active state) / 10. Skip to end
///  11. Speed pill (popup menu)
///  12. Time display (monospace, MM:SS / MM:SS)
///  13. Spacer
///  14. Split / 15. Delete (error color when a clip is selected)
///  16. Divider
///  17. Export button (accent gradient + accent glow)
class EditorToolbar extends ConsumerWidget {
  const EditorToolbar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);
    final hasSelection = editorState.selectedClipId != null;

    return Container(
      height: 56,
      decoration: const BoxDecoration(
        color: AppTheme.surface,
        border: Border(bottom: BorderSide(color: AppTheme.border)),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Row(
          children: [
            // ── 1. Back ─────────────────────────────────────────────
            SvgToolbarButton(
              icon: AppIcons.back,
              tooltip: 'Back to projects',
              onPressed: () => context.go('/'),
            ),
            const SizedBox(width: 4),

            // ── 2. Project name + resolution badge ─────────────────
            Text(
              project?.name ?? 'Untitled',
              style: context.textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(width: 8),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: AppTheme.primaryLight.withOpacity(0.15),
                borderRadius: BorderRadius.circular(AppTheme.radiusFull),
              ),
              child: Text(
                '${project?.width ?? 1920}×${project?.height ?? 1080}',
                style: context.textTheme.labelSmall?.copyWith(
                  color: AppTheme.primaryLight,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),

            // ── 3. Divider ─────────────────────────────────────────
            const SizedBox(width: 12),
            _verticalDivider(),
            const SizedBox(width: 12),

            // ── 4. Undo / 5. Redo / 6. Save ────────────────────────
            SvgToolbarButton(
              icon: AppIcons.undo,
              tooltip: 'Undo',
              onPressed: () => ref.read(editorProvider.notifier).undo(),
            ),
            SvgToolbarButton(
              icon: AppIcons.redo,
              tooltip: 'Redo',
              onPressed: () => ref.read(editorProvider.notifier).redo(),
            ),
            SvgToolbarButton(
              icon: AppIcons.save,
              tooltip: 'Save',
              color: AppTheme.primaryLight,
              onPressed: () async {
                await ref
                    .read(projectProvider.notifier)
                    .saveCurrentProject();
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Project saved'),
                      duration: Duration(seconds: 1),
                    ),
                  );
                }
              },
            ),

            // ── 7. Divider ─────────────────────────────────────────
            const SizedBox(width: 8),
            _verticalDivider(),
            const SizedBox(width: 8),

            // ── 8. Skip to start ───────────────────────────────────
            SvgToolbarButton(
              icon: AppIcons.skipBack,
              tooltip: 'Go to start',
              onPressed: () => ref.read(editorProvider.notifier).seekTo(0),
            ),
            // ── 9. Play / Pause (active state highlights with primary bg)
            SvgToolbarButton(
              icon: editorState.isPlaying ? AppIcons.pause : AppIcons.play,
              tooltip: editorState.isPlaying ? 'Pause' : 'Play',
              isActive: editorState.isPlaying,
              onPressed: () =>
                  ref.read(editorProvider.notifier).togglePlayback(),
            ),
            // ── 10. Skip to end ────────────────────────────────────
            SvgToolbarButton(
              icon: AppIcons.skipForward,
              tooltip: 'Go to end',
              onPressed: () => ref
                  .read(editorProvider.notifier)
                  .seekTo(editorState.durationMs),
            ),

            // ── 11. Speed pill ─────────────────────────────────────
            _PlaybackSpeedButton(
              currentSpeed: editorState.playbackSpeed,
              onSpeedChanged: (speed) =>
                  ref.read(editorProvider.notifier).setPlaybackSpeed(speed),
            ),

            const SizedBox(width: 8),

            // ── 12. Time display ───────────────────────────────────
            Container(
              padding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: AppTheme.surfaceVariant,
                borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                border: Border.all(color: AppTheme.border, width: 1),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    Duration(milliseconds: editorState.currentTimeMs)
                        .shortFormatted,
                    style: context.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: AppTheme.textPrimary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  Text(
                    ' / ',
                    style: context.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: AppTheme.textDisabled,
                    ),
                  ),
                  Text(
                    Duration(milliseconds: editorState.durationMs)
                        .shortFormatted,
                    style: context.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),

            // ── 13. Spacer ─────────────────────────────────────────
            const Spacer(),

            // ── 14. Split ──────────────────────────────────────────
            SvgToolbarButton(
              icon: AppIcons.split,
              tooltip: 'Split at playhead',
              onPressed: () =>
                  ref.read(editorProvider.notifier).splitAtPlayhead(),
            ),
            // ── 15. Delete (error color when selection exists) ─────
            SvgToolbarButton(
              icon: AppIcons.deleteItem,
              tooltip: 'Delete selected',
              color: hasSelection ? AppTheme.error : null,
              onPressed: hasSelection
                  ? () => ref.read(editorProvider.notifier).deleteSelected()
                  : null,
            ),

            // ── 16. Divider ────────────────────────────────────────
            const SizedBox(width: 8),
            _verticalDivider(),
            const SizedBox(width: 8),

            // ── 17. Export button (accent gradient + glow) ─────────
            _ExportButton(
              onPressed: project != null
                  ? () => context.go('/export/${project.id}')
                  : null,
            ),

            const SizedBox(width: 4),
          ],
        ),
      ),
    );
  }

  /// Standard vertical group divider used across the toolbar.
  static Widget _verticalDivider() =>
      Container(width: 1, height: 24, color: AppTheme.border);
}

/// Export button — pill shape with `accentGradient` background and
/// `accentGlow` shadow.
class _ExportButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const _ExportButton({this.onPressed});

  @override
  Widget build(BuildContext context) {
    final enabled = onPressed != null;
    return Opacity(
      opacity: enabled ? 1.0 : 0.5,
      child: Container(
        decoration: BoxDecoration(
          gradient: AppTheme.accentGradient,
          borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
          boxShadow: enabled ? AppTheme.accentGlow() : null,
        ),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: onPressed,
            borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
            child: Padding(
              padding:
                  const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  SvgPicture.asset(
                    AppIcons.exportIcon,
                    width: 16,
                    height: 16,
                    colorFilter: const ColorFilter.mode(
                      Colors.white,
                      BlendMode.srcIn,
                    ),
                  ),
                  const SizedBox(width: 8),
                  const Text(
                    'Export',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                      fontFamily: 'Inter',
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Playback speed selector — pill button that opens a popup menu.
class _PlaybackSpeedButton extends StatelessWidget {
  final double currentSpeed;
  final ValueChanged<double> onSpeedChanged;

  const _PlaybackSpeedButton({
    required this.currentSpeed,
    required this.onSpeedChanged,
  });

  static const _speeds = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0];

  @override
  Widget build(BuildContext context) {
    final isNonDefault = currentSpeed != 1.0;
    return PopupMenuButton<double>(
      offset: const Offset(0, 44),
      tooltip: 'Playback speed',
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: BoxDecoration(
          color: isNonDefault
              ? AppTheme.primary.withOpacity(0.2)
              : AppTheme.surfaceVariant,
          borderRadius: BorderRadius.circular(AppTheme.radiusFull),
          border: Border.all(
            color: isNonDefault ? AppTheme.primary : AppTheme.border,
            width: 1,
          ),
        ),
        child: Text(
          '${currentSpeed.toStringAsFixed(1)}x',
          style: context.textTheme.labelSmall?.copyWith(
            color: isNonDefault
                ? AppTheme.primaryLight
                : AppTheme.textSecondary,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      itemBuilder: (context) => _speeds.map((speed) {
        return PopupMenuItem<double>(
          value: speed,
          child: Row(
            children: [
              if (speed == currentSpeed)
                const Icon(Icons.check, size: 16, color: AppTheme.primary)
              else
                const SizedBox(width: 16),
              const SizedBox(width: 8),
              Text('${speed}x'),
            ],
          ),
        );
      }).toList(),
      onSelected: onSpeedChanged,
    );
  }
}
