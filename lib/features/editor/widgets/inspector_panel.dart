import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';

/// Inspector panel - Shows properties of the selected clip or track
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
        if (selectedClip != null) break;
      }
    }

    // If a track is selected (but no clip), show track properties
    if (selectedClip == null && editorState.selectedTrackId != null && project != null) {
      for (final track in project.tracks) {
        if (track.id == editorState.selectedTrackId) {
          return _buildTrackInspector(context, ref, track);
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

          // Track volume (show for audio and video tracks)
          if (selectedTrack != null) ...[
            _SectionHeader(title: 'Track Volume'),
            const SizedBox(height: 8),
            _VolumeControl(
              volume: selectedTrack.volume,
              isMuted: !selectedTrack.visible,
              onVolumeChanged: (value) {
                // TODO: Wire to engine set_track_volume
              },
              onMuteToggled: () {
                // TODO: Wire to engine toggle_track_visibility
              },
            ),
            const SizedBox(height: 16),

            // Audio ducking (show for audio tracks)
            if (selectedTrack.trackType == TrackType.audio) ...[
              _SectionHeader(title: 'Audio Ducking'),
              const SizedBox(height: 8),
              _DuckingControl(
                enabled: false,
                duckLevel: 0.3,
                onEnabledChanged: (enabled) {
                  // TODO: Wire to engine set_ducking
                },
                onLevelChanged: (level) {
                  // TODO: Wire to engine set_ducking
                },
              ),
              const SizedBox(height: 16),
            ],
          ],

          // Effects section
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

  /// Build inspector for a selected track (no clip selected)
  Widget _buildTrackInspector(BuildContext context, WidgetRef ref, TrackModel track) {
    return Container(
      color: AppTheme.surface,
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Header
          Row(
            children: [
              Text('Track Inspector', style: context.textTheme.titleMedium),
              const Spacer(),
              IconButton(
                onPressed: () => ref.read(editorProvider.notifier).selectTrack(null),
                icon: const Icon(Icons.close, size: 18),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Track type badge
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _trackColor(track.trackType).withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              _trackTypeName(track.trackType),
              style: context.textTheme.labelMedium?.copyWith(
                color: _trackColor(track.trackType),
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Track name
          _SectionHeader(title: 'Name'),
          const SizedBox(height: 8),
          Text(track.name, style: context.textTheme.bodyMedium),
          const SizedBox(height: 16),

          // Clip count
          _PropertyRow(label: 'Clips', value: '${track.clips.length}'),
          _PropertyRow(label: 'Locked', value: track.locked ? 'Yes' : 'No'),
          _PropertyRow(label: 'Visible', value: track.visible ? 'Yes' : 'No'),
          const SizedBox(height: 16),

          // Volume control
          _SectionHeader(title: 'Volume'),
          const SizedBox(height: 8),
          _VolumeControl(
            volume: track.volume,
            isMuted: !track.visible,
            onVolumeChanged: (value) {
              // TODO: Wire to engine set_track_volume
            },
            onMuteToggled: () {
              // TODO: Wire to engine toggle_track_visibility
            },
          ),
          const SizedBox(height: 16),

          // Audio ducking (for audio tracks)
          if (track.trackType == TrackType.audio) ...[
            _SectionHeader(title: 'Audio Ducking'),
            const SizedBox(height: 8),
            _DuckingControl(
              enabled: false,
              duckLevel: 0.3,
              onEnabledChanged: (enabled) {
                // TODO: Wire to engine set_ducking
              },
              onLevelChanged: (level) {
                // TODO: Wire to engine set_ducking
              },
            ),
          ],
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

/// Volume control widget with mute toggle
class _VolumeControl extends StatelessWidget {
  final double volume;
  final bool isMuted;
  final ValueChanged<double> onVolumeChanged;
  final VoidCallback onMuteToggled;

  const _VolumeControl({
    required this.volume,
    required this.isMuted,
    required this.onVolumeChanged,
    required this.onMuteToggled,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            // Mute button
            IconButton(
              onPressed: onMuteToggled,
              icon: Icon(
                isMuted ? Icons.volume_off : Icons.volume_up,
                size: 20,
                color: isMuted ? AppTheme.error : AppTheme.textSecondary,
              ),
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            ),
            const SizedBox(width: 8),

            // Volume slider
            Expanded(
              child: Slider(
                value: volume,
                min: 0.0,
                max: 2.0,
                divisions: 20,
                onChanged: onVolumeChanged,
              ),
            ),
            const SizedBox(width: 8),

            // Volume label
            SizedBox(
              width: 48,
              child: Text(
                isMuted ? 'MUTE' : '${(volume * 100).round()}%',
                style: context.textTheme.labelSmall?.copyWith(
                  color: isMuted ? AppTheme.error : AppTheme.textSecondary,
                  fontFamily: 'monospace',
                ),
                textAlign: TextAlign.right,
              ),
            ),
          ],
        ),
        const SizedBox(height: 4),

        // Quick volume presets
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            _VolumePresetChip(label: '0%', volume: 0.0, current: volume, onChanged: onVolumeChanged),
            _VolumePresetChip(label: '50%', volume: 0.5, current: volume, onChanged: onVolumeChanged),
            _VolumePresetChip(label: '100%', volume: 1.0, current: volume, onChanged: onVolumeChanged),
            _VolumePresetChip(label: '150%', volume: 1.5, current: volume, onChanged: onVolumeChanged),
          ],
        ),
      ],
    );
  }
}

class _VolumePresetChip extends StatelessWidget {
  final String label;
  final double volume;
  final double current;
  final ValueChanged<double> onChanged;

  const _VolumePresetChip({
    required this.label,
    required this.volume,
    required this.current,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final isSelected = (current - volume).abs() < 0.05;
    return InkWell(
      onTap: () => onChanged(volume),
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: isSelected ? AppTheme.primary.withValues(alpha: 0.2) : Colors.transparent,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(
            color: isSelected ? AppTheme.primary : AppTheme.textDisabled.withValues(alpha: 0.3),
            width: 1,
          ),
        ),
        child: Text(
          label,
          style: context.textTheme.labelSmall?.copyWith(
            color: isSelected ? AppTheme.primary : AppTheme.textDisabled,
            fontSize: 10,
          ),
        ),
      ),
    );
  }
}

/// Ducking control widget with enable toggle and level slider
class _DuckingControl extends StatelessWidget {
  final bool enabled;
  final double duckLevel;
  final ValueChanged<bool> onEnabledChanged;
  final ValueChanged<double> onLevelChanged;

  const _DuckingControl({
    required this.enabled,
    required this.duckLevel,
    required this.onEnabledChanged,
    required this.onLevelChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Enable toggle
        Row(
          children: [
            Switch(
              value: enabled,
              onChanged: onEnabledChanged,
              activeColor: AppTheme.primary,
              materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                enabled ? 'Ducking Active' : 'Enable Ducking',
                style: context.textTheme.bodySmall?.copyWith(
                  color: enabled ? AppTheme.primary : AppTheme.textDisabled,
                ),
              ),
            ),
          ],
        ),

        if (enabled) ...[
          const SizedBox(height: 8),
          Text(
            'When this track plays, other tracks reduce to ${(duckLevel * 100).round()}% volume',
            style: context.textTheme.bodySmall?.copyWith(
              color: AppTheme.textSecondary,
              fontSize: 11,
            ),
          ),
          const SizedBox(height: 8),

          // Duck level slider
          Row(
            children: [
              const Icon(Icons.volume_down, size: 16, color: AppTheme.textDisabled),
              Expanded(
                child: Slider(
                  value: duckLevel,
                  min: 0.0,
                  max: 0.8,
                  divisions: 8,
                  onChanged: onLevelChanged,
                ),
              ),
              const Icon(Icons.volume_up, size: 16, color: AppTheme.textDisabled),
              const SizedBox(width: 8),
              SizedBox(
                width: 40,
                child: Text(
                  '${(duckLevel * 100).round()}%',
                  style: context.textTheme.labelSmall?.copyWith(
                    fontFamily: 'monospace',
                  ),
                  textAlign: TextAlign.right,
                ),
              ),
            ],
          ),

          // Quick presets
          const SizedBox(height: 4),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              _DuckPresetChip(label: 'Soft', level: 0.4, current: duckLevel, onChanged: onLevelChanged),
              _DuckPresetChip(label: 'Medium', level: 0.25, current: duckLevel, onChanged: onLevelChanged),
              _DuckPresetChip(label: 'Deep', level: 0.1, current: duckLevel, onChanged: onLevelChanged),
            ],
          ),
        ],
      ],
    );
  }
}

class _DuckPresetChip extends StatelessWidget {
  final String label;
  final double level;
  final double current;
  final ValueChanged<double> onChanged;

  const _DuckPresetChip({
    required this.label,
    required this.level,
    required this.current,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final isSelected = (current - level).abs() < 0.02;
    return InkWell(
      onTap: () => onChanged(level),
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
        decoration: BoxDecoration(
          color: isSelected ? AppTheme.secondary.withValues(alpha: 0.2) : Colors.transparent,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(
            color: isSelected ? AppTheme.secondary : AppTheme.textDisabled.withValues(alpha: 0.3),
            width: 1,
          ),
        ),
        child: Text(
          label,
          style: context.textTheme.labelSmall?.copyWith(
            color: isSelected ? AppTheme.secondary : AppTheme.textDisabled,
            fontSize: 10,
          ),
        ),
      ),
    );
  }
}
