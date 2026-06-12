import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../../core/services/engine_service.dart';
import '../providers/editor_provider.dart';
import '../../projects/providers/project_provider.dart';

/// Top toolbar for the editor screen
///
/// Phase 4 additions:
/// - Save button (saves to .epp format via engine)
/// - Playback speed control
class EditorToolbar extends ConsumerWidget {
  const EditorToolbar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);

    return Container(
      height: 52,
      decoration: const BoxDecoration(
        color: AppTheme.surface,
        border: Border(bottom: BorderSide(color: Color(0xFF2A2A3E))),
      ),
      child: Row(
        children: [
          // Back button + Project name
          IconButton(
            onPressed: () => context.go('/'),
            icon: const Icon(Icons.arrow_back, size: 20),
            tooltip: 'Back to projects',
          ),
          const SizedBox(width: 4),
          Text(
            project?.name ?? 'Untitled',
            style: context.textTheme.titleSmall,
          ),
          const SizedBox(width: 4),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: AppTheme.primary.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              '${project?.width ?? 1920}x${project?.height ?? 1080}',
              style: context.textTheme.labelSmall?.copyWith(
                color: AppTheme.primaryLight,
              ),
            ),
          ),

          const Spacer(),

          // Undo/Redo
          _ToolbarIconButton(
            icon: Icons.undo,
            tooltip: 'Undo',
            onPressed: () => ref.read(editorProvider.notifier).undo(),
          ),
          _ToolbarIconButton(
            icon: Icons.redo,
            tooltip: 'Redo',
            onPressed: () => ref.read(editorProvider.notifier).redo(),
          ),

          const SizedBox(width: 8),

          // Save button
          _ToolbarIconButton(
            icon: Icons.save,
            tooltip: 'Save project',
            highlightColor: AppTheme.primary,
            onPressed: () async {
              await ref.read(projectProvider.notifier).saveCurrentProject();
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

          const SizedBox(width: 8),

          // Playback controls
          _ToolbarIconButton(
            icon: Icons.skip_previous,
            tooltip: 'Go to start',
            onPressed: () => ref.read(editorProvider.notifier).seekTo(0),
          ),
          _ToolbarIconButton(
            icon: editorState.isPlaying ? Icons.pause : Icons.play_arrow,
            tooltip: editorState.isPlaying ? 'Pause' : 'Play',
            onPressed: () => ref.read(editorProvider.notifier).togglePlayback(),
            highlightColor: AppTheme.primary,
          ),
          _ToolbarIconButton(
            icon: Icons.skip_next,
            tooltip: 'Go to end',
            onPressed: () => ref.read(editorProvider.notifier).seekTo(editorState.durationMs),
          ),

          // Playback speed
          const SizedBox(width: 4),
          _PlaybackSpeedButton(
            currentSpeed: editorState.playbackSpeed,
            onSpeedChanged: (speed) => ref.read(editorProvider.notifier).setPlaybackSpeed(speed),
          ),

          const SizedBox(width: 8),

          // Time display
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: AppTheme.surfaceVariant,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  Duration(milliseconds: editorState.currentTimeMs).formatted,
                  style: context.textTheme.bodySmall?.copyWith(
                    fontFamily: 'monospace',
                    color: AppTheme.textPrimary,
                  ),
                ),
                Text(
                  ' / ',
                  style: context.textTheme.bodySmall?.copyWith(
                    color: AppTheme.textDisabled,
                  ),
                ),
                Text(
                  Duration(milliseconds: editorState.durationMs).formatted,
                  style: context.textTheme.bodySmall?.copyWith(
                    fontFamily: 'monospace',
                    color: AppTheme.textSecondary,
                  ),
                ),
              ],
            ),
          ),

          const Spacer(),

          // Split & Delete
          _ToolbarIconButton(
            icon: Icons.content_cut,
            tooltip: 'Split at playhead',
            onPressed: () => ref.read(editorProvider.notifier).splitAtPlayhead(),
          ),
          _ToolbarIconButton(
            icon: Icons.delete_outline,
            tooltip: 'Delete selected',
            onPressed: editorState.selectedClipId != null
                ? () => ref.read(editorProvider.notifier).deleteSelected()
                : null,
          ),

          const SizedBox(width: 8),

          // Export button
          ElevatedButton.icon(
            onPressed: project != null
                ? () => context.go('/export/${project.id}')
                : null,
            icon: const Icon(Icons.file_download, size: 16),
            label: const Text('Export'),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              minimumSize: Size.zero,
            ),
          ),

          const SizedBox(width: 12),
        ],
      ),
    );
  }
}

class _ToolbarIconButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback? onPressed;
  final Color? highlightColor;

  const _ToolbarIconButton({
    required this.icon,
    required this.tooltip,
    this.onPressed,
    this.highlightColor,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      onPressed: onPressed,
      icon: Icon(icon, size: 20),
      tooltip: tooltip,
      style: IconButton.styleFrom(
        foregroundColor: highlightColor ?? AppTheme.textSecondary,
        minimumSize: const Size(36, 36),
        padding: EdgeInsets.zero,
      ),
    );
  }
}

/// Playback speed selector button — shows a popup menu with speed options.
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
    return PopupMenuButton<double>(
      offset: const Offset(0, 36),
      tooltip: 'Playback speed',
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: currentSpeed != 1.0
              ? AppTheme.primary.withValues(alpha: 0.2)
              : Colors.transparent,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(
            color: currentSpeed != 1.0
                ? AppTheme.primary
                : const Color(0xFF2A2A3E),
            width: 1,
          ),
        ),
        child: Text(
          '${currentSpeed}x',
          style: context.textTheme.labelSmall?.copyWith(
            color: currentSpeed != 1.0 ? AppTheme.primaryLight : AppTheme.textSecondary,
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
