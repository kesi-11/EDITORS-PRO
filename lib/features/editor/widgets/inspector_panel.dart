import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';

/// Inspector panel - Shows properties of the selected clip
class InspectorPanel extends ConsumerWidget {
  const InspectorPanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);

    // Find the selected clip
    ClipModel? selectedClip;
    TrackModel? selectedTrack;
    if (editorState.selectedClipId != null && project != null) {
      for (final track in project.tracks) {
        for (final clip in track.clips) {
          if (clip.id == editorState.selectedClipId) {
            selectedClip = clip;
            selectedTrack = track;
            break;
          }
        }
      }
    }

    if (!editorState.showInspector || selectedClip == null) {
      return _buildEmptyInspector(context);
    }

    return Container(
      color: AppTheme.surface,
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Header
          Row(
            children: [
              Text('Inspector', style: context.textTheme.titleMedium),
              const Spacer(),
              IconButton(
                onPressed: () => ref.read(editorProvider.notifier).selectClip(null),
                icon: const Icon(Icons.close, size: 18),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Clip type badge
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _trackColor(selectedTrack?.trackType).withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              _trackTypeName(selectedTrack?.trackType),
              style: context.textTheme.labelMedium?.copyWith(
                color: _trackColor(selectedTrack?.trackType),
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Timing section
          _SectionHeader(title: 'Timing'),
          const SizedBox(height: 8),
          _PropertyRow(label: 'Start', value: Duration(milliseconds: selectedClip.startMs).formatted),
          _PropertyRow(label: 'Duration', value: Duration(milliseconds: selectedClip.durationMs).formatted),
          _PropertyRow(label: 'Trim Start', value: '${selectedClip.trimStartMs}ms'),
          _PropertyRow(label: 'Trim End', value: '${selectedClip.trimEndMs}ms'),
          const SizedBox(height: 16),

          // Speed control
          _SectionHeader(title: 'Speed'),
          const SizedBox(height: 8),
          _SpeedControl(
            speed: selectedClip.speed,
            onChanged: (value) {
              // Will call engine to update clip speed
            },
          ),
          const SizedBox(height: 16),

          // Opacity control
          _SectionHeader(title: 'Opacity'),
          const SizedBox(height: 8),
          _OpacityControl(
            opacity: selectedClip.opacity,
            onChanged: (value) {
              // Will call engine to update clip opacity
            },
          ),
          const SizedBox(height: 16),

          // Effects section (for future)
          _SectionHeader(title: 'Effects'),
          const SizedBox(height: 8),
          Center(
            child: Text(
              'Add effects to this clip',
              style: context.textTheme.bodySmall,
            ),
          ),
          const SizedBox(height: 8),
          OutlinedButton.icon(
            onPressed: () {
              // Switch to effects panel
              ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.effects);
            },
            icon: const Icon(Icons.add, size: 16),
            label: const Text('Add Effect'),
            style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(36)),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyInspector(BuildContext context) {
    return Container(
      color: AppTheme.surface,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.touch_app, size: 48, color: AppTheme.textDisabled),
            const SizedBox(height: 16),
            Text(
              'Select a clip',
              style: context.textTheme.titleSmall?.copyWith(
                color: AppTheme.textDisabled,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Tap any clip on the timeline\nto view its properties',
              style: context.textTheme.bodySmall,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Color _trackColor(TrackType? type) {
    switch (type) {
      case TrackType.video: return AppTheme.videoTrackColor;
      case TrackType.audio: return AppTheme.audioTrackColor;
      case TrackType.text: return AppTheme.textTrackColor;
      case TrackType.effect: return AppTheme.effectTrackColor;
      default: return AppTheme.textSecondary;
    }
  }

  String _trackTypeName(TrackType? type) {
    switch (type) {
      case TrackType.video: return 'VIDEO CLIP';
      case TrackType.audio: return 'AUDIO CLIP';
      case TrackType.text: return 'TEXT CLIP';
      case TrackType.effect: return 'EFFECT';
      default: return 'CLIP';
    }
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;

  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Text(
      title.toUpperCase(),
      style: context.textTheme.labelMedium?.copyWith(
        color: AppTheme.textDisabled,
        letterSpacing: 1,
      ),
    );
  }
}

class _PropertyRow extends StatelessWidget {
  final String label;
  final String value;

  const _PropertyRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: context.textTheme.bodySmall),
          Text(value, style: context.textTheme.bodySmall?.copyWith(
            fontFamily: 'monospace',
            color: AppTheme.textPrimary,
          )),
        ],
      ),
    );
  }
}

class _SpeedControl extends StatelessWidget {
  final double speed;
  final ValueChanged<double> onChanged;

  const _SpeedControl({required this.speed, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text('${speed.toStringAsFixed(1)}x', style: context.textTheme.titleSmall),
            Wrap(
              spacing: 6,
              children: [0.25, 0.5, 1.0, 1.5, 2.0, 4.0].map((s) {
                final isSelected = (speed - s).abs() < 0.01;
                return ChoiceChip(
                  label: Text('${s}x', style: const TextStyle(fontSize: 10)),
                  selected: isSelected,
                  onSelected: (_) => onChanged(s),
                  visualDensity: VisualDensity.compact,
                );
              }).toList(),
            ),
          ],
        ),
        Slider(
          value: speed,
          min: 0.1,
          max: 8.0,
          onChanged: onChanged,
        ),
      ],
    );
  }
}

class _OpacityControl extends StatelessWidget {
  final double opacity;
  final ValueChanged<double> onChanged;

  const _OpacityControl({required this.opacity, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text('${(opacity * 100).round()}%', style: context.textTheme.titleSmall),
          ],
        ),
        Slider(
          value: opacity,
          min: 0.0,
          max: 1.0,
          onChanged: onChanged,
        ),
      ],
    );
  }
}
