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
                ref.read(editorProvider.notifier).setTrackVolume(selectedTrack!.id, value);
              },
              onMuteToggled: () {
                ref.read(editorProvider.notifier).toggleTrackVisibility(selectedTrack!.id);
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
                  ref.read(editorProvider.notifier).setDucking(
                    selectedTrack!.id,
                    enabled: enabled,
                    duckLevel: 0.3,
                  );
                },
                onLevelChanged: (level) {
                  ref.read(editorProvider.notifier).setDucking(
                    selectedTrack!.id,
                    enabled: true,
                    duckLevel: level,
                  );
                },
              ),
              const SizedBox(height: 16),
            ],
          ],

          // Effects section
          _SectionHeader(title: 'Effects'),
          const SizedBox(height: 8),
          _EffectsSection(clipId: selectedClip.id),
          const SizedBox(height: 16),

          // Transitions section
          _SectionHeader(title: 'Transitions'),
          const SizedBox(height: 8),
          _TransitionsSection(clipId: selectedClip.id),
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
              ref.read(editorProvider.notifier).setTrackVolume(track.id, value);
            },
            onMuteToggled: () {
              ref.read(editorProvider.notifier).toggleTrackVisibility(track.id);
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
                ref.read(editorProvider.notifier).setDucking(
                  track.id,
                  enabled: enabled,
                  duckLevel: 0.3,
                );
              },
              onLevelChanged: (level) {
                ref.read(editorProvider.notifier).setDucking(
                  track.id,
                  enabled: true,
                  duckLevel: level,
                );
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
      ],
    );
  }
}

/// Ducking control widget
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
                  style: context.textTheme.labelSmall?.copyWith(fontFamily: 'monospace'),
                  textAlign: TextAlign.right,
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }
}

/// Effects section — shows applied effects with parameter sliders
class _EffectsSection extends ConsumerStatefulWidget {
  final String clipId;

  const _EffectsSection({required this.clipId});

  @override
  ConsumerState<_EffectsSection> createState() => _EffectsSectionState();
}

class _EffectsSectionState extends ConsumerState<_EffectsSection> {
  List<Map<String, dynamic>> _effects = [];

  void _updateEffectsFromTimeline() {
    // Effects come through the timeline state, so we read them from
    // the project provider's track/clip data. In a future iteration,
    // we can use a dedicated effectsProvider for more granular updates.
    final project = ref.read(currentProjectProvider);
    if (project == null) return;

    for (final track in project.tracks) {
      for (final clip in track.clips) {
        if (clip.id == widget.clipId) {
          // The clip model may not have effects yet if the bridge
          // codegen hasn't run. Use empty list as fallback.
          setState(() {
            _effects = [];
          });
          return;
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (_effects.isEmpty) ...[
          Center(
            child: Text(
              'No effects applied',
              style: context.textTheme.bodySmall?.copyWith(
                color: AppTheme.textDisabled,
              ),
            ),
          ),
        ],

        // Applied effects list
        ..._effects.map((effect) => _AppliedEffectCard(
          effect: effect,
          onParameterChanged: (paramName, value) {
            final effectId = effect['id'] as String? ?? '';
            ref.read(editorProvider.notifier).setEffectParameter(
              effectId, paramName, value,
            );
          },
          onToggleEnabled: () {
            final effectId = effect['id'] as String? ?? '';
            ref.read(editorProvider.notifier).toggleEffect(effectId);
          },
          onRemove: () {
            final effectId = effect['id'] as String? ?? '';
            ref.read(editorProvider.notifier).removeEffect(effectId);
          },
        )),

        const SizedBox(height: 8),

        // Add Effect button
        OutlinedButton.icon(
          onPressed: () {
            ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.effects);
          },
          icon: const Icon(Icons.add, size: 16),
          label: const Text('Add Effect'),
          style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(36)),
        ),
      ],
    );
  }
}

/// Card showing an applied effect with its parameter sliders
class _AppliedEffectCard extends StatelessWidget {
  final Map<String, dynamic> effect;
  final void Function(String paramName, double value) onParameterChanged;
  final VoidCallback onToggleEnabled;
  final VoidCallback onRemove;

  const _AppliedEffectCard({
    required this.effect,
    required this.onParameterChanged,
    required this.onToggleEnabled,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    final name = effect['name'] as String? ?? 'Unknown';
    final enabled = effect['enabled'] as bool? ?? true;
    final parameters = effect['parameters'] as List<dynamic>? ?? [];

    return Card(
      color: AppTheme.surfaceVariant,
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Effect header
            Row(
              children: [
                Icon(
                  Icons.auto_fix_high,
                  size: 16,
                  color: enabled ? AppTheme.primary : AppTheme.textDisabled,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    name,
                    style: context.textTheme.bodyMedium?.copyWith(
                      color: enabled ? AppTheme.textPrimary : AppTheme.textDisabled,
                    ),
                  ),
                ),
                // Toggle enabled
                IconButton(
                  onPressed: onToggleEnabled,
                  icon: Icon(
                    enabled ? Icons.visibility : Icons.visibility_off,
                    size: 16,
                    color: enabled ? AppTheme.textSecondary : AppTheme.textDisabled,
                  ),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                ),
                // Remove
                IconButton(
                  onPressed: onRemove,
                  icon: Icon(
                    Icons.delete_outline,
                    size: 16,
                    color: AppTheme.error.withValues(alpha: 0.7),
                  ),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                ),
              ],
            ),

            // Parameter sliders
            if (enabled && parameters.isNotEmpty) ...[
              const SizedBox(height: 4),
              ...parameters.map((param) {
                final p = param as Map<dynamic, dynamic>;
                final paramName = p['name'] as String? ?? '';
                final displayName = p['display_name'] as String? ?? paramName;
                final value = (p['value'] as num?)?.toDouble() ?? 0.0;
                final minVal = (p['min_value'] as num?)?.toDouble() ?? 0.0;
                final maxVal = (p['max_value'] as num?)?.toDouble() ?? 1.0;
                final step = (p['step'] as num?)?.toDouble() ?? 0.01;

                return _EffectParameterSlider(
                  name: displayName,
                  value: value,
                  min: minVal,
                  max: maxVal,
                  step: step,
                  onChanged: (v) => onParameterChanged(paramName, v),
                );
              }),
            ],
          ],
        ),
      ),
    );
  }
}

/// Single parameter slider for an effect
class _EffectParameterSlider extends StatelessWidget {
  final String name;
  final double value;
  final double min;
  final double max;
  final double step;
  final ValueChanged<double> onChanged;

  const _EffectParameterSlider({
    required this.name,
    required this.value,
    required this.min,
    required this.max,
    required this.step,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    // Format display value
    String displayValue;
    if (step >= 1.0) {
      displayValue = value.round().toString();
    } else if (step >= 0.1) {
      displayValue = value.toStringAsFixed(1);
    } else {
      displayValue = value.toStringAsFixed(2);
    }

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              name,
              style: context.textTheme.labelSmall?.copyWith(
                color: AppTheme.textSecondary,
              ),
            ),
          ),
          Expanded(
            child: Slider(
              value: value.clamp(min, max),
              min: min,
              max: max,
              onChanged: onChanged,
            ),
          ),
          SizedBox(
            width: 40,
            child: Text(
              displayValue,
              style: context.textTheme.labelSmall?.copyWith(
                fontFamily: 'monospace',
                color: AppTheme.textPrimary,
              ),
              textAlign: TextAlign.right,
            ),
          ),
        ],
      ),
    );
  }
}

/// Transitions section — shows applied transitions and add button
class _TransitionsSection extends ConsumerWidget {
  final String clipId;

  const _TransitionsSection({required this.clipId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Center(
          child: Text(
            'Add transitions between clips',
            style: context.textTheme.bodySmall?.copyWith(
              color: AppTheme.textDisabled,
            ),
          ),
        ),
        const SizedBox(height: 8),
        OutlinedButton.icon(
          onPressed: () {
            _showTransitionPicker(context, ref);
          },
          icon: const Icon(Icons.swap_horiz, size: 16),
          label: const Text('Add Transition'),
          style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(36)),
        ),
      ],
    );
  }

  void _showTransitionPicker(BuildContext context, WidgetRef ref) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: AppTheme.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (context) => DraggableScrollableSheet(
        initialChildSize: 0.6,
        minChildSize: 0.3,
        maxChildSize: 0.9,
        expand: false,
        builder: (context, scrollController) => Column(
          children: [
            // Handle
            Container(
              width: 40,
              height: 4,
              margin: const EdgeInsets.symmetric(vertical: 8),
              decoration: BoxDecoration(
                color: AppTheme.textDisabled,
                borderRadius: BorderRadius.circular(2),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Text('Transitions', style: context.textTheme.titleMedium),
                  const Spacer(),
                  IconButton(
                    onPressed: () => Navigator.pop(context),
                    icon: const Icon(Icons.close, size: 18),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            // Transition picker content
            const Expanded(
              child: TransitionPicker(),
            ),
          ],
        ),
      ),
    );
  }
}
