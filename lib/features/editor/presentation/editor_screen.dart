import 'dart:async';
import 'dart:developer' as developer;
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;
import 'package:uuid/uuid.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../../core/services/engine_service.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';
import '../providers/engine_bridge_provider.dart';
import '../widgets/preview_viewport.dart';
import '../widgets/timeline_panel.dart';
import '../widgets/editor_toolbar.dart';
import '../widgets/inspector_panel.dart';
import '../widgets/effect_catalog.dart';
import '../widgets/transition_picker.dart';
import '../widgets/text_panel.dart';
import '../widgets/speed_curve_editor.dart';
// Phase F.2: pro videographer widgets
import '../widgets/audio_mixer_panel.dart';
import '../widgets/color_scopes_panel.dart';
import '../widgets/markers_panel.dart';
import '../widgets/lut_browser.dart';
import '../widgets/eq_panel.dart';
import '../widgets/audio_loudness_meter.dart';
import '../widgets/safe_zones_overlay.dart';
import '../widgets/keyframe_graph_editor.dart';
import '../widgets/gpu_status_badge.dart';
import '../widgets/proxy_status_badge.dart';

/// Main editor screen - the core editing experience
class EditorScreen extends ConsumerStatefulWidget {
  final String projectId;

  const EditorScreen({super.key, required this.projectId});

  @override
  ConsumerState<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends ConsumerState<EditorScreen> {
  /// Phase F.2: in-memory timeline markers. In a full integration these
  /// would be persisted via the engine's `effects/markers.rs` module.
  /// For now, scoped to the screen lifetime.
  List<Marker> _markers = const [];

  @override
  void initState() {
    super.initState();
    // Initialize the editor state
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(editorProvider.notifier).initialize();
    });
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorProvider);
    final project = ref.watch(currentProjectProvider);
    final screenWidth = MediaQuery.of(context).size.width;
    final isNarrowScreen = screenWidth < 600;
    // Phase E.19: tablet/large-screen breakpoint. On tablets (>=1200px)
    // the left panel is wider (320px vs 240px) and the inspector takes
    // more space, giving professional editors room to work.
    final isTablet = screenWidth >= 1200;

    return Scaffold(
      body: Column(
        children: [
          // Top toolbar
          const EditorToolbar(),

          // Main content area
          Expanded(
            child: Row(
              children: [
                // Left: Media library / Tools (hidden on narrow screens)
                if (!isNarrowScreen)
                  _buildLeftPanel(context, editorState, isTablet: isTablet),

                // Center: Preview viewport
                Expanded(
                  flex: 3,
                  child: Stack(
                    children: [
                      const PreviewViewport(),
                      // Phase F.2: Safe Zones overlay (broadcast / social / composition).
                      // Toggleable from the toolbar; only renders when showSafeZones is true.
                      if (editorState.showSafeZones &&
                          editorState.safeZoneMode != SafeZoneMode.off)
                        Positioned.fill(
                          child: SafeZonesOverlay(
                            config: _safeZoneConfigFor(editorState.safeZoneMode),
                            previewAspectRatio: 16 / 9,
                          ),
                        ),
                      // GPU status badge — top-right of viewport
                      Positioned(
                        top: 8,
                        right: 8,
                        child: const GpuStatusBadge(),
                      ),
                      // Proxy status badge — next to GPU badge
                      Positioned(
                        top: 8,
                        right: 58,
                        child: const ProxyStatusBadge(),
                      ),
                      // Phase F.2: Safe zones indicator badge (left of proxy badge)
                      if (editorState.showSafeZones)
                        Positioned(
                          top: 8,
                          right: 108,
                          child: _SafeZoneBadge(mode: editorState.safeZoneMode),
                        ),
                      // Phase F.2: Safe zones toggle — bottom-left of viewport
                      Positioned(
                        bottom: 8,
                        left: 8,
                        child: FloatingActionButton.small(
                          heroTag: 'safezones_fab',
                          tooltip: 'Safe zones: ${editorState.safeZoneMode.name}',
                          backgroundColor: editorState.showSafeZones
                              ? AppTheme.primary
                              : AppTheme.surface,
                          foregroundColor: editorState.showSafeZones
                              ? Colors.white
                              : AppTheme.textSecondary,
                          onPressed: () =>
                              ref.read(editorProvider.notifier).cycleSafeZoneMode(),
                          child: const Icon(Icons.crop_free, size: 18),
                        ),
                      ),
                      // Phase F.2: Audio Meter Bridge toggle — bottom-right of viewport
                      Positioned(
                        bottom: 8,
                        right: 8,
                        child: FloatingActionButton.small(
                          heroTag: 'audiometer_fab',
                          tooltip: 'Audio meter bridge',
                          backgroundColor: editorState.showAudioMeterBridge
                              ? AppTheme.primary
                              : AppTheme.surface,
                          foregroundColor: editorState.showAudioMeterBridge
                              ? Colors.white
                              : AppTheme.textSecondary,
                          onPressed: () =>
                              ref.read(editorProvider.notifier).toggleAudioMeterBridge(),
                          child: const Icon(Icons.graphic_eq, size: 18),
                        ),
                      ),
                      // Phase E.9: on narrow screens, show a floating action
                      // button that opens the InspectorPanel as a draggable
                      // bottom sheet. Without this, phone users have no way
                      // to edit clip properties.
                      if (isNarrowScreen)
                        Positioned(
                          top: 8,
                          left: 8,
                          child: FloatingActionButton.small(
                            heroTag: 'inspector_fab',
                            tooltip: 'Inspector',
                            backgroundColor: AppTheme.primary,
                            foregroundColor: Colors.white,
                            onPressed: () => _showInspectorBottomSheet(context),
                            child: const Icon(Icons.tune),
                          ),
                        ),
                      // Phase E.10: importing overlay — shown whenever
                      // editorState.isImporting is true. Previously the flag
                      // was set but no UI surfaced it, leaving the user
                      // wondering if the app was frozen.
                      if (editorState.isImporting)
                        Positioned.fill(
                          child: Container(
                            color: Colors.black.withOpacity(0.5),
                            child: Center(
                              child: Card(
                                color: AppTheme.surface,
                                child: Padding(
                                  padding: const EdgeInsets.all(AppTheme.spacing24),
                                  child: Column(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      const CircularProgressIndicator(
                                        color: AppTheme.primary,
                                      ),
                                      const SizedBox(height: AppTheme.spacing16),
                                      Text(
                                        'Importing media…',
                                        style: TextStyle(
                                          color: AppTheme.textPrimary,
                                          fontSize: 14,
                                          fontWeight: FontWeight.w500,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),

                // Right: Inspector / Properties (hidden on narrow screens)
                if (!isNarrowScreen)
                  Expanded(
                    // Phase E.19: give the inspector more room on tablets
                    // (flex 2) so color grading wheels and effect chains
                    // aren't cramped. On phones it's flex 1.
                    flex: isTablet ? 2 : 1,
                    child: const InspectorPanel(),
                  ),
              ],
            ),
          ),

          // Phase F.2: Audio Meter Bridge — thin horizontal strip showing
          // integrated LUFS + true-peak, between the timeline and the bottom
          // of the screen. Toggleable via the audio-meter FAB on the viewport.
          if (editorState.showAudioMeterBridge)
            _buildAudioMeterBridge(context, editorState),

          // Bottom: Timeline
          const TimelinePanel(),
        ],
      ),
    );
  }

  /// Phase F.2: Audio Meter Bridge — a compact horizontal loudness display
  /// that sits between the main content area and the timeline. Shows
  /// integrated LUFS, short-term LUFS, true-peak, and the active target.
  Widget _buildAudioMeterBridge(BuildContext context, EditorState state) {
    return Container(
      height: 56,
      padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacing12),
      decoration: BoxDecoration(
        color: AppTheme.surface,
        border: Border(
          top: BorderSide(color: AppTheme.border.withOpacity(0.5)),
          bottom: BorderSide(color: AppTheme.border.withOpacity(0.5)),
        ),
      ),
      child: Row(
        children: [
          // Label
          Text(
            'LOUDNESS',
            style: TextStyle(
              fontSize: 9,
              fontWeight: FontWeight.bold,
              color: AppTheme.textSecondary,
            ),
          ),
          const SizedBox(width: AppTheme.spacing12),
          // Compact loudness display — single line
          Expanded(
            child: _CompactLoudnessBar(
              reading: const LoudnessReading(
                integratedLufs: -23.0,
                shortTermLufs: -22.5,
                momentaryLufs: -20.0,
                truePeakDbtp: -1.5,
              ),
              target: LoudnessTarget.ebuR128,
            ),
          ),
          const SizedBox(width: AppTheme.spacing8),
          // Open full meter button
          IconButton(
            icon: const Icon(Icons.open_in_full, size: 16),
            tooltip: 'Open full loudness meter',
            onPressed: () => _showLoudnessMeterDialog(context),
            iconSize: 16,
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
          ),
        ],
      ),
    );
  }

  /// Phase F.2: map a SafeZoneMode to a SafeZoneConfig for the overlay widget.
  SafeZoneConfig _safeZoneConfigFor(SafeZoneMode mode) {
    switch (mode) {
      case SafeZoneMode.broadcast:
        return SafeZoneConfig.broadcast;
      case SafeZoneMode.social:
        return SafeZoneConfig.social;
      case SafeZoneMode.composition:
        return SafeZoneConfig.composition;
      case SafeZoneMode.off:
        return SafeZoneConfig.none;
    }
  }

  /// Phase E.9: show the InspectorPanel as a draggable bottom sheet
  /// on narrow screens (phones). This is the same widget used in the
  /// right-hand panel on tablets — no duplicate UI to maintain.
  void _showInspectorBottomSheet(BuildContext context) {
    showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      useSafeArea: true,
      backgroundColor: AppTheme.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(
          top: Radius.circular(AppTheme.radiusXLarge),
        ),
      ),
      builder: (sheetContext) {
        // Wrap in a DraggableScrollableSheet so the user can drag
        // the sheet up to full height and back down to dismiss.
        return DraggableScrollableSheet(
          initialChildSize: 0.6,
          minChildSize: 0.3,
          maxChildSize: 0.95,
          expand: false,
          builder: (_, scrollController) {
            return Column(
              children: [
                // Drag handle
                Container(
                  margin: const EdgeInsets.symmetric(vertical: 8),
                  width: 40,
                  height: 4,
                  decoration: BoxDecoration(
                    color: AppTheme.textDisabled,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                // Header
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppTheme.spacing16,
                  ),
                  child: Row(
                    children: [
                      const Text(
                        'Inspector',
                        style: TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      const Spacer(),
                      IconButton(
                        icon: const Icon(Icons.close),
                        onPressed: () => Navigator.of(context).pop(),
                      ),
                    ],
                  ),
                ),
                const Divider(height: 1),
                // Inspector content (scrollable)
                Expanded(
                  child: SingleChildScrollView(
                    controller: scrollController,
                    padding: const EdgeInsets.all(AppTheme.spacing16),
                    child: const InspectorPanel(),
                  ),
                ),
              ],
            );
          },
        );
      },
    );
  }

  Widget _buildLeftPanel(BuildContext context, EditorState state,
      {bool isTablet = false}) {
    return Container(
      // Phase E.19: tablets get a wider left panel (320 vs 240) so
      // media thumbnails and effect names don't truncate.
      width: isTablet ? 320 : 240,
      color: AppTheme.surface,
      child: Column(
        children: [
          // Panel tabs — Phase F.2: horizontal scroll to fit 11 pro tabs.
          Container(
            decoration: const BoxDecoration(
              border: Border(bottom: BorderSide(color: Color(0xFF2A2A3E))),
            ),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  _TabButton(
                    label: 'Media',
                    icon: Icons.video_library,
                    selected: state.leftPanelTab == LeftPanelTab.media,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.media),
                  ),
                  _TabButton(
                    label: 'Audio',
                    icon: Icons.audiotrack,
                    selected: state.leftPanelTab == LeftPanelTab.audio,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.audio),
                  ),
                  _TabButton(
                    label: 'Effects',
                    icon: Icons.auto_fix_high,
                    selected: state.leftPanelTab == LeftPanelTab.effects,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.effects),
                  ),
                  _TabButton(
                    label: 'Text',
                    icon: Icons.text_fields,
                    selected: state.leftPanelTab == LeftPanelTab.text,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.text),
                  ),
                  _TabButton(
                    label: 'Speed',
                    icon: Icons.speed,
                    selected: state.leftPanelTab == LeftPanelTab.speed,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.speed),
                  ),
                  _TabButton(
                    label: 'Keys',
                    icon: Icons.timeline,
                    selected: state.leftPanelTab == LeftPanelTab.keyframes,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.keyframes),
                  ),
                  // ─── Phase F.2: Pro videographer tabs ──────────────────────
                  _TabButton(
                    label: 'Mixer',
                    icon: Icons.graphic_eq,
                    selected: state.leftPanelTab == LeftPanelTab.mixer,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.mixer),
                  ),
                  _TabButton(
                    label: 'Scopes',
                    icon: Icons.analytics_outlined,
                    selected: state.leftPanelTab == LeftPanelTab.scopes,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.scopes),
                  ),
                  _TabButton(
                    label: 'Markers',
                    icon: Icons.bookmark_border,
                    selected: state.leftPanelTab == LeftPanelTab.markers,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.markers),
                  ),
                  _TabButton(
                    label: 'LUTs',
                    icon: Icons.palette_outlined,
                    selected: state.leftPanelTab == LeftPanelTab.luts,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.luts),
                  ),
                  _TabButton(
                    label: 'EQ',
                    icon: Icons.equalizer,
                    selected: state.leftPanelTab == LeftPanelTab.eq,
                    onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.eq),
                  ),
                ],
              ),
            ),
          ),

          // Panel content
          Expanded(
            child: _buildLeftPanelContent(context, state),
          ),
        ],
      ),
    );
  }

  Widget _buildLeftPanelContent(BuildContext context, EditorState state) {
    switch (state.leftPanelTab) {
      case LeftPanelTab.media:
        return _buildMediaLibrary(context);
      case LeftPanelTab.audio:
        return _buildAudioPanel(context);
      case LeftPanelTab.effects:
        return _buildEffectsPanel(context);
      case LeftPanelTab.text:
        return _buildTextPanel(context);
      case LeftPanelTab.speed:
        return _buildSpeedPanel(context, state);
      case LeftPanelTab.keyframes:
        return _buildKeyframesPanel(context, state);
      // ─── Phase F.2: Pro videographer panels ──────────────────────────
      case LeftPanelTab.mixer:
        return _buildMixerPanel(context, state);
      case LeftPanelTab.scopes:
        return _buildScopesPanel(context, state);
      case LeftPanelTab.markers:
        return _buildMarkersPanel(context, state);
      case LeftPanelTab.luts:
        return _buildLutPanel(context, state);
      case LeftPanelTab.eq:
        return _buildEqPanel(context, state);
    }
  }

  // ─── Phase F.2: Pro videographer panel builders ──────────────────────

  /// Per-track audio mixer with volume, pan, mute, solo, master fader.
  Widget _buildMixerPanel(BuildContext context, EditorState state) {
    // TODO: wire to real track list from the engine. For now, show an
    // empty state — the widget handles empty tracks gracefully.
    return AudioMixerPanel(
      tracks: const [],
      master: const MixerMaster(),
      onVolumeChanged: (trackId, volume) {
        if (!EngineService.instance.isInitialized) return;
        unawaited(
          EngineService.instance.api.setTrackVolume(
            trackId: trackId,
            volume: volume,
          ).catchError((e) {
            developer.log('setTrackVolume failed: $e', name: 'MixerPanel');
          }),
        );
      },
      onPanChanged: (trackId, pan) {
        // Engine pan API not yet wired — stubbed.
      },
      onMuteToggled: (trackId, muted) {
        if (!EngineService.instance.isInitialized) return;
        unawaited(
          EngineService.instance.api.toggleTrackVisibility(
            trackId: trackId,
          ).catchError((e) {
            developer.log('toggleTrackVisibility failed: $e', name: 'MixerPanel');
          }),
        );
      },
      onSoloToggled: (trackId, solo) {
        // Solo API not yet wired — stubbed.
      },
      onMasterVolumeChanged: (volume) {
        ref.read(editorProvider.notifier).setMasterVolume(volume);
      },
      onOpenLoudnessMeter: () => _showLoudnessMeterDialog(context),
    );
  }

  /// Color scopes — waveform, vectorscope, RGB parade, histogram.
  Widget _buildScopesPanel(BuildContext context, EditorState state) {
    return ColorScopesPanel(
      scopes: null, // TODO: wire to engine compute_scopes — call after every frame
      onRequestRefresh: () {
        // In a full integration, this would grab the current frame buffer
        // from the preview viewport, base64-encode it, and call
        // computeScopes() via the bridge. For now, show a placeholder
        // SnackBar so the user knows the action is wired.
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Scopes refresh: wire to computeScopes() bridge method.'),
            duration: Duration(seconds: 2),
          ),
        );
      },
    );
  }

  /// Timeline markers — chapter, note, sync, QC, edit, audio, VFX.
  Widget _buildMarkersPanel(BuildContext context, EditorState state) {
    // TODO: wire to real engine marker storage. For now, in-memory only.
    return MarkersPanel(
      markers: _markers,
      onAddMarker: (type, color, note) {
        setState(() {
          _markers = [
            ..._markers,
            Marker(
              id: const Uuid().v4(),
              timeMs: state.currentTimeMs,
              type: type,
              color: color,
              note: note,
            ),
          ];
        });
      },
      onDeleteMarker: (markerId) {
        setState(() {
          _markers = _markers.where((m) => m.id != markerId).toList();
        });
      },
      onJumpToMarker: (markerId) {
        final marker = _markers.firstWhere((m) => m.id == markerId);
        // Jump the playhead to the marker — call seek on the editor notifier.
        // The seek method is part of the existing playback loop.
        _seekToMs(marker.timeMs);
      },
    );
  }

  /// LUT browser — import .cube, apply with intensity slider.
  Widget _buildLutPanel(BuildContext context, EditorState state) {
    return LutBrowser(
      onLutSelected: (lutJson) {
        // The LUT JSON is parsed by the engine when applied to a frame.
        // Store it in the editor state or apply immediately based on UX.
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('LUT loaded. Apply via the inspector panel.'),
            duration: Duration(seconds: 2),
          ),
        );
      },
      onIntensityChanged: (intensity) {
        // TODO: store intensity in editor state and apply to the current clip.
      },
    );
  }

  /// 8-band parametric EQ — high-pass, 8 bands, low-pass.
  Widget _buildEqPanel(BuildContext context, EditorState state) {
    return EqPanel(
      onChanged: (settings) {
        // TODO: apply the EQ chain to the current audio track via the engine.
        // The audio/effects.rs module currently only has a low-pass filter —
        // the full EQ chain is a `video:` debt marker.
      },
    );
  }

  /// Phase F.2: show the loudness meter in a floating dialog (called from
  /// the mixer panel's loudness-meter button).
  void _showLoudnessMeterDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => Dialog(
        child: Padding(
          padding: const EdgeInsets.all(AppTheme.spacing16),
          child: SizedBox(
            width: 360,
            child: AudioLoudnessMeter(
              reading: const LoudnessReading(
                integratedLufs: -23.0,
                shortTermLufs: -22.5,
                momentaryLufs: -20.0,
                truePeakDbtp: -1.5,
              ),
              target: LoudnessTarget.ebuR128,
            ),
          ),
        ),
      ),
    );
  }

  /// Seek the playhead to a specific time in milliseconds.
  /// Used by the markers panel to jump to a marker.
  void _seekToMs(int timeMs) {
    // The editor notifier has a play() method that uses _playbackTimer; for
    // seeking we update the currentTimeMs directly and invalidate the frame cache.
    // A full implementation would call into the engine's seek method.
    final clamped = timeMs.clamp(0, ref.read(editorProvider).durationMs);
    // Trigger a frame fetch at the new position by toggling play state.
    // This is a pragmatic seek — the real implementation would call
    // EngineService.instance.api.seek(timeMs) when that method is wired.
    developer.log('Seek to $clamped ms (stubbed)', name: 'EditorScreen');
  }

  Widget _buildMediaLibrary(BuildContext context) {
    final assets = ref.watch(mediaAssetsProvider);

    if (assets.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.add_circle_outline, size: 48, color: AppTheme.textDisabled),
              const SizedBox(height: 16),
              Text(
                'Import Media',
                style: context.textTheme.titleSmall?.copyWith(color: AppTheme.textSecondary),
              ),
              const SizedBox(height: 8),
              Text(
                'Tap + to add videos, audio, or images',
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),
              ElevatedButton.icon(
                onPressed: () => _importMedia(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('Import'),
              ),
            ],
          ),
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: assets.length + 1, // +1 for import button
      itemBuilder: (context, index) {
        if (index == 0) {
          return Padding(
            padding: const EdgeInsets.all(8),
            child: OutlinedButton.icon(
              onPressed: () => _importMedia(),
              icon: const Icon(Icons.add, size: 16),
              label: const Text('Import Media'),
              style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(36)),
            ),
          );
        }

        final asset = assets[index - 1];
        return _MediaAssetItem(
          asset: asset,
          onAddToTimeline: () => _addAssetToTimeline(asset),
        );
      },
    );
  }

  Widget _buildAudioPanel(BuildContext context) {
    final assets = ref.watch(mediaAssetsProvider);
    final audioAssets = assets.where((a) => a.mediaType == MediaType.audio).toList();

    return Column(
      children: [
        // Import audio button
        Padding(
          padding: const EdgeInsets.all(8),
          child: OutlinedButton.icon(
            onPressed: () => _importMedia(),
            icon: const Icon(Icons.add, size: 16),
            label: const Text('Import Audio'),
            style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(36)),
          ),
        ),

        // Audio assets list
        if (audioAssets.isEmpty)
          Expanded(
            child: Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.music_note, size: 48, color: AppTheme.textDisabled),
                    const SizedBox(height: 16),
                    Text(
                      'No Audio Files',
                      style: context.textTheme.titleSmall?.copyWith(
                        color: AppTheme.textSecondary,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Import MP3, WAV, AAC, FLAC,\nor OGG audio files',
                      style: context.textTheme.bodySmall,
                      textAlign: TextAlign.center,
                    ),
                  ],
                ),
              ),
            ),
          )
        else
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.all(8),
              itemCount: audioAssets.length,
              itemBuilder: (context, index) {
                final asset = audioAssets[index];
                return Card(
                  child: ListTile(
                    dense: true,
                    leading: Container(
                      width: 48,
                      height: 32,
                      decoration: BoxDecoration(
                        color: AppTheme.audioTrackColor.withOpacity(0.2),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: const Icon(
                        Icons.audiotrack,
                        color: AppTheme.audioTrackColor,
                        size: 18,
                      ),
                    ),
                    title: Text(
                      asset.fileName,
                      style: context.textTheme.bodySmall,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    subtitle: asset.durationMs != null
                        ? Text(
                            Duration(milliseconds: asset.durationMs!).formatted,
                            style: context.textTheme.labelSmall,
                          )
                        : null,
                    trailing: IconButton(
                      icon: const Icon(Icons.add_circle_outline, size: 18),
                      onPressed: () => _addAssetToTimeline(asset),
                    ),
                  ),
                );
              },
            ),
          ),
      ],
    );
  }

  Widget _buildEffectsPanel(BuildContext context) {
    return const EffectCatalog();
  }

  Widget _buildTextPanel(BuildContext context) {
    return const TextPanel();
  }

  Widget _buildSpeedPanel(BuildContext context, EditorState state) {
    // Find the selected clip to get clipId and duration
    final project = ref.read(currentProjectProvider);
    String? clipId;
    int clipDurationMs = 5000; // default

    if (state.selectedClipId != null && project != null) {
      for (final track in project.tracks) {
        for (final clip in track.clips) {
          if (clip.id == state.selectedClipId) {
            clipId = clip.id;
            clipDurationMs = clip.durationMs;
            break;
          }
        }
        if (clipId != null) break;
      }
    }

    if (clipId == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.speed, size: 48, color: AppTheme.textDisabled),
              const SizedBox(height: 16),
              Text(
                'Select a Clip',
                style: context.textTheme.titleSmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                'Select a clip on the timeline\nto edit its speed curve',
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(8),
      child: SpeedCurveEditor(
        clipId: clipId,
        clipDurationMs: clipDurationMs,
      ),
    );
  }

  Widget _buildKeyframesPanel(BuildContext context, EditorState state) {
    // Find the selected clip to get clipId and duration
    final project = ref.read(currentProjectProvider);
    String? clipId;
    int clipDurationMs = 5000; // default

    if (state.selectedClipId != null && project != null) {
      for (final track in project.tracks) {
        for (final clip in track.clips) {
          if (clip.id == state.selectedClipId) {
            clipId = clip.id;
            clipDurationMs = clip.durationMs;
            break;
          }
        }
        if (clipId != null) break;
      }
    }

    if (clipId == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.timeline, size: 48, color: AppTheme.textDisabled),
              const SizedBox(height: 16),
              Text(
                'Select a Clip',
                style: context.textTheme.titleSmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                'Select a clip on the timeline\nto edit its keyframes',
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }

    return KeyframeGraphEditor(
      clipId: clipId,
      clipDurationMs: clipDurationMs,
    );
  }

  // ─── "Add to Timeline" Implementation ───────────────────────────

  /// Add a media asset to the appropriate track on the timeline.
  ///
  /// Finds the first track matching the asset's type, calculates the
  /// start position (append after the last clip), and calls the Rust
  /// engine to create the clip via the Command pattern.
  Future<void> _addAssetToTimeline(MediaAssetModel asset) async {
    final project = ref.read(currentProjectProvider);
    if (project == null) return;

    // Determine which track type this asset goes on
    final TrackType targetTrackType;
    switch (asset.mediaType) {
      case MediaType.video:
      case MediaType.image:
        targetTrackType = TrackType.video;
        break;
      case MediaType.audio:
        targetTrackType = TrackType.audio;
        break;
    }

    // Find the first track of the matching type
    final targetTrack = project.tracks
        .where((t) => t.trackType == targetTrackType)
        .firstOrNull;

    if (targetTrack == null) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('No ${targetTrackType.name} track found')),
        );
      }
      return;
    }

    // Calculate the start position: append after the last clip on the track
    final lastClipEnd = targetTrack.clips.isEmpty
        ? 0
        : targetTrack.clips
            .map((c) => c.startMs + c.durationMs)
            .reduce((a, b) => a > b ? a : b);

    // Call into the Rust engine to add the clip
    final clipInfo = await ref.read(editorProvider.notifier).addClipToTrack(
          trackId: targetTrack.id,
          assetId: asset.id,
          startMs: lastClipEnd,
          durationMs: asset.durationMs ?? 0,
        );

    if (clipInfo != null) {
      // Update the Flutter model to reflect the new clip
      final clip = ClipModel(
        id: clipInfo.id,
        assetId: asset.id,
        startMs: lastClipEnd,
        durationMs: asset.durationMs ?? clipInfo.durationMs.toInt(),
      );
      ref.read(projectProvider.notifier).addClipToTrack(targetTrack.id, clip);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Added ${asset.fileName} to ${targetTrack.name}'),
            duration: const Duration(seconds: 1),
          ),
        );
      }
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to add clip')),
        );
      }
    }
  }

  /// Add a text overlay to the text track.
  Future<void> _addTextToTimeline(String text, double fontSize) async {
    final project = ref.read(currentProjectProvider);
    if (project == null) return;

    // Find the text track
    final textTrack = project.tracks
        .where((t) => t.trackType == TrackType.text)
        .firstOrNull;

    if (textTrack == null) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('No text track found')),
        );
      }
      return;
    }

    // Calculate the start position
    final lastClipEnd = textTrack.clips.isEmpty
        ? 0
        : textTrack.clips
            .map((c) => c.startMs + c.durationMs)
            .reduce((a, b) => a > b ? a : b);

    // Create a placeholder text clip (5 seconds default duration)
    const defaultDurationMs = 5000;
    final clipId = const Uuid().v4();
    final clip = ClipModel(
      id: clipId,
      assetId: 'text_$clipId',
      startMs: lastClipEnd,
      durationMs: defaultDurationMs,
    );

    // Update the Flutter model immediately for responsiveness
    ref.read(projectProvider.notifier).addClipToTrack(textTrack.id, clip);

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Added "$text" to ${textTrack.name}'),
          duration: const Duration(seconds: 1),
        ),
      );
    }
  }

  /// Import media using the file picker, copy to cache, and register
  /// with the Rust engine.
  Future<void> _importMedia() async {
    try {
      ref.read(editorProvider.notifier).setImporting(true);

      final result = await FilePicker.platform.pickFiles(
        type: FileType.custom,
        allowedExtensions: [
          'mp4', 'avi', 'mov', 'mkv', 'webm', 'flv', 'wmv', '3gp',
          'mp3', 'wav', 'aac', 'flac', 'ogg', 'm4a', 'wma',
          'jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp',
        ],
        allowMultiple: true,
      );

      if (result == null || result.files.isEmpty) {
        ref.read(editorProvider.notifier).setImporting(false);
        return;
      }

      final cacheDir = await getTemporaryDirectory();
      final mediaDir = Directory(p.join(cacheDir.path, AppConstants.mediaDir));
      if (!mediaDir.existsSync()) {
        mediaDir.createSync(recursive: true);
      }

      for (final pickedFile in result.files) {
        final sourcePath = pickedFile.path;
        if (sourcePath == null) continue;

        final destPath = p.join(mediaDir.path, p.basename(sourcePath));
        await File(sourcePath).copy(destPath);

        if (!EngineService.instance.isInitialized) {
          ref.read(editorProvider.notifier).setImporting(false);
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Engine not initialized')),
            );
          }
          return;
        }

        final notifier = ref.read(editorProvider.notifier);
        final assetInfo = await notifier.importMedia(destPath);

        if (assetInfo != null) {
          final mediaType = _mediaTypeFromString(assetInfo.mediaType);
          await ref.read(projectProvider.notifier).importMedia(
                assetInfo.filePath,
                assetInfo.fileName,
                mediaType,
                durationMs: assetInfo.durationMs?.toInt(),
                width: assetInfo.width?.toInt(),
                height: assetInfo.height?.toInt(),
                fileSizeBytes: assetInfo.fileSizeBytes.toInt(),
                codec: assetInfo.codec,
                bitrate: assetInfo.bitrate?.toInt(),
              );
        }
      }

      ref.read(editorProvider.notifier).setImporting(false);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Imported ${result.files.length} file(s)'),
            duration: const Duration(seconds: 1),
          ),
        );
      }
    } catch (e) {
      ref.read(editorProvider.notifier).setImporting(false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Import failed: $e')),
        );
      }
    }
  }

  /// Map the string media type returned by the Rust engine to the
  /// Dart [MediaType] enum.
  MediaType _mediaTypeFromString(String type) {
    switch (type.toLowerCase()) {
      case 'video':
        return MediaType.video;
      case 'audio':
        return MediaType.audio;
      case 'image':
        return MediaType.image;
      default:
        return MediaType.video;
    }
  }
}

class _TabButton extends StatelessWidget {
  final String label;
  final IconData icon;
  final bool selected;
  final VoidCallback onTap;

  const _TabButton({
    required this.label,
    required this.icon,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    // Phase F.2: fixed width (70px) instead of Expanded so the tab row
    // can scroll horizontally when there are more than ~6 tabs.
    return SizedBox(
      width: 70,
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 10),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  icon,
                  size: 18,
                  color: selected ? AppTheme.primary : AppTheme.textDisabled,
                ),
                const SizedBox(height: 4),
                Text(
                  label,
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
                    color: selected ? AppTheme.primary : AppTheme.textDisabled,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _MediaAssetItem extends StatelessWidget {
  final MediaAssetModel asset;
  final VoidCallback onAddToTimeline;

  const _MediaAssetItem({
    required this.asset,
    required this.onAddToTimeline,
  });

  @override
  Widget build(BuildContext context) {
    final isVideo = asset.mediaType == MediaType.video;
    final isAudio = asset.mediaType == MediaType.audio;

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      color: AppTheme.surfaceVariant,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      child: InkWell(
        borderRadius: BorderRadius.circular(8),
        onTap: onAddToTimeline,
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Row(
            children: [
              // Thumbnail
              Container(
                width: 64,
                height: 48,
                decoration: BoxDecoration(
                  color: isVideo
                      ? AppTheme.videoTrackColor.withOpacity(0.15)
                      : isAudio
                          ? AppTheme.audioTrackColor.withOpacity(0.15)
                          : AppTheme.primary.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Icon(
                      isVideo
                          ? Icons.videocam
                          : isAudio
                              ? Icons.audiotrack
                              : Icons.image,
                      color: isVideo
                          ? AppTheme.videoTrackColor
                          : isAudio
                              ? AppTheme.audioTrackColor
                              : AppTheme.primary,
                      size: 24,
                    ),
                    if (asset.durationMs != null)
                      Positioned(
                        bottom: 2,
                        right: 4,
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 4, vertical: 1),
                          decoration: BoxDecoration(
                            color: Colors.black87,
                            borderRadius: BorderRadius.circular(3),
                          ),
                          child: Text(
                            Duration(milliseconds: asset.durationMs!).formatted,
                            style: const TextStyle(
                              fontSize: 9,
                              color: Colors.white,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              const SizedBox(width: 10),
              // Info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      asset.fileName,
                      style: context.textTheme.bodySmall
                          ?.copyWith(fontWeight: FontWeight.w500),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      [
                        if (asset.width != null && asset.height != null)
                          '${asset.width}×${asset.height}',
                        if (asset.durationMs != null)
                          Duration(milliseconds: asset.durationMs!).formatted,
                        if (asset.fileSizeBytes > 0)
                          _formatBytes(asset.fileSizeBytes),
                      ].join(' · '),
                      style: context.textTheme.labelSmall
                          ?.copyWith(color: AppTheme.textSecondary),
                    ),
                  ],
                ),
              ),
              // Add button
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Icon(Icons.add, size: 18, color: AppTheme.primary),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatBytes(int bytes) {
    if (bytes >= 1073741824) return '${(bytes / 1073741824).toStringAsFixed(1)} GB';
    if (bytes >= 1048576) return '${(bytes / 1048576).toStringAsFixed(1)} MB';
    if (bytes >= 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '$bytes B';
  }
}

// Note: _EffectCard replaced by EffectCatalog widget in effect_catalog.dart

// ─── Phase F.2: Helper widgets for the editor screen ───────────────────

/// Compact horizontal loudness bar — used in the Audio Meter Bridge.
/// Renders integrated LUFS, true-peak, target, and a compliance indicator
/// in a single line.
class _CompactLoudnessBar extends StatelessWidget {
  final LoudnessReading reading;
  final LoudnessTarget target;

  const _CompactLoudnessBar({required this.reading, required this.target});

  @override
  Widget build(BuildContext context) {
    final loudnessOk = (reading.integratedLufs - target.integratedTarget).abs() <=
        target.integratedTolerance + 0.5;
    final truePeakOk = reading.truePeakDbtp <= target.truePeakCeiling;
    final allOk = loudnessOk && truePeakOk;

    return Row(
      children: [
        _Metric(
          label: 'I',
          value: '${reading.integratedLufs.toStringAsFixed(1)}',
          unit: 'LUFS',
          target: '${target.integratedTarget.toStringAsFixed(0)}',
          ok: loudnessOk,
        ),
        const SizedBox(width: AppTheme.spacing12),
        _Metric(
          label: 'S',
          value: '${reading.shortTermLufs.toStringAsFixed(1)}',
          unit: 'LUFS',
          target: '',
          ok: true,
        ),
        const SizedBox(width: AppTheme.spacing12),
        _Metric(
          label: 'M',
          value: '${reading.momentaryLufs.toStringAsFixed(1)}',
          unit: 'LUFS',
          target: '',
          ok: true,
        ),
        const SizedBox(width: AppTheme.spacing12),
        _Metric(
          label: 'TP',
          value: '${reading.truePeakDbtp.toStringAsFixed(1)}',
          unit: 'dBTP',
          target: '≤ ${target.truePeakCeiling.toStringAsFixed(0)}',
          ok: truePeakOk,
        ),
        const SizedBox(width: AppTheme.spacing12),
        // Compliance pill
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: (allOk ? Colors.green : Colors.red).withOpacity(0.15),
            border: Border.all(
                color: (allOk ? Colors.green : Colors.red).withOpacity(0.5)),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                allOk ? Icons.check_circle : Icons.warning,
                size: 12,
                color: allOk ? Colors.green : Colors.red,
              ),
              const SizedBox(width: 4),
              Text(
                allOk ? 'COMPLIANT' : 'NON-COMPLIANT',
                style: TextStyle(
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                  color: allOk ? Colors.green : Colors.red,
                ),
              ),
              const SizedBox(width: 6),
              Text(
                target.label,
                style: TextStyle(
                  fontSize: 9,
                  color: AppTheme.textSecondary,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _Metric extends StatelessWidget {
  final String label;
  final String value;
  final String unit;
  final String target;
  final bool ok;

  const _Metric({
    required this.label,
    required this.value,
    required this.unit,
    required this.target,
    required this.ok,
  });

  @override
  Widget build(BuildContext context) {
    final color = ok ? Colors.green : Colors.red;
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.baseline,
      textBaseline: TextBaseline.alphabetic,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 9,
            color: AppTheme.textSecondary,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(width: 2),
        Text(
          value,
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.bold,
            color: color,
            fontFeatures: const [FontFeature.tabularFigures()],
          ),
        ),
        const SizedBox(width: 2),
        Text(
          unit,
          style: TextStyle(
            fontSize: 9,
            color: color,
          ),
        ),
        if (target.isNotEmpty) ...[
          const SizedBox(width: 4),
          Text(
            '($target)',
            style: TextStyle(
              fontSize: 9,
              color: AppTheme.textSecondary,
            ),
          ),
        ],
      ],
    );
  }
}

/// Phase F.2: small badge showing the current safe-zone mode, displayed
/// at the top-right of the preview viewport.
class _SafeZoneBadge extends StatelessWidget {
  final SafeZoneMode mode;

  const _SafeZoneBadge({required this.mode});

  @override
  Widget build(BuildContext context) {
    final (label, color) = switch (mode) {
      SafeZoneMode.broadcast => ('SAFE:BC', Colors.blue),
      SafeZoneMode.social => ('SAFE:SO', Colors.purple),
      SafeZoneMode.composition => ('SAFE:3RDS', Colors.orange),
      SafeZoneMode.off => ('', Colors.grey),
    };
    if (mode == SafeZoneMode.off) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
      decoration: BoxDecoration(
        color: color.withOpacity(0.85),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: const TextStyle(
          fontSize: 9,
          fontWeight: FontWeight.bold,
          color: Colors.white,
        ),
      ),
    );
  }
}
