import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../providers/editor_provider.dart';

/// Video preview viewport - displays the current frame
class PreviewViewport extends ConsumerStatefulWidget {
  const PreviewViewport({super.key});

  @override
  ConsumerState<PreviewViewport> createState() => _PreviewViewportState();
}

class _PreviewViewportState extends ConsumerState<PreviewViewport> {
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
                      // In production, this renders frames from the Rust engine
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
    // When the Rust engine is connected, this displays actual video frames
    // For now, show a placeholder with project info
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
