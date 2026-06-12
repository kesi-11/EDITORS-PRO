import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../providers/editor_provider.dart';
import '../../projects/providers/project_provider.dart';

/// Top toolbar for the editor screen
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
