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
import '../widgets/keyframe_graph_editor.dart';
import '../widgets/gpu_status_badge.dart';
import '../widgets/proxy_status_badge.dart';
import '../widgets/audio_mixer_panel.dart';
import '../widgets/video_scopes.dart';
import '../widgets/markers_panel.dart';
import '../widgets/lut_browser.dart';
import '../widgets/audio_eq_panel.dart';

/// Main editor screen - the core editing experience
class EditorScreen extends ConsumerStatefulWidget {
  final String projectId;

  const EditorScreen({super.key, required this.projectId});

  @override
  ConsumerState<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends ConsumerState<EditorScreen> {
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

          // Bottom: Audio Meter Bridge
          const SizedBox(
            height: 28,
            child: AudioMeterBridge(),
          ),
          // Bottom: Timeline
          const TimelinePanel(),
        ],
      ),
    );
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
          // Panel tabs
          Container(
            decoration: const BoxDecoration(
              border: Border(bottom: BorderSide(color: Color(0xFF2A2A3E))),
            ),
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
                _TabButton(
                  label: 'Mixer',
                  icon: Icons.sliders,
                  selected: state.leftPanelTab == LeftPanelTab.mixer,
                  onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.mixer),
                ),
                _TabButton(
                  label: 'Scopes',
                  icon: Icons.waveform,
                  selected: state.leftPanelTab == LeftPanelTab.scopes,
                  onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.scopes),
                ),
                _TabButton(
                  label: 'Markers',
                  icon: Icons.bookmark,
                  selected: state.leftPanelTab == LeftPanelTab.markers,
                  onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.markers),
                ),
                _TabButton(
                  label: 'LUTs',
                  icon: Icons.palette,
                  selected: state.leftPanelTab == LeftPanelTab.luts,
                  onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.luts),
                ),
                _TabButton(
                  label: 'EQ',
                  icon: Icons.graphic_eq,
                  selected: state.leftPanelTab == LeftPanelTab.eq,
                  onTap: () => ref.read(editorProvider.notifier).setLeftPanelTab(LeftPanelTab.eq),
                ),
              ],
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
      case LeftPanelTab.mixer:
        return const AudioMixerPanel();
      case LeftPanelTab.scopes:
        return const VideoScopesPanel();
      case LeftPanelTab.markers:
        return const MarkersPanel();
      case LeftPanelTab.luts:
        return _buildLutPanel(context, state);
      case LeftPanelTab.eq:
        return _buildEqPanel(context, state);
    }
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
    return Expanded(
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
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildLutPanel(BuildContext context, EditorState state) {
    if (state.selectedClipId == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.palette, size: 48, color: AppTheme.textDisabled),
              const SizedBox(height: 16),
              Text(
                'Select a Clip',
                style: context.textTheme.titleSmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                'Select a clip on the timeline\nto apply a LUT',
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }
    return LutBrowser(clipId: state.selectedClipId!);
  }

  Widget _buildEqPanel(BuildContext context, EditorState state) {
    // Find the selected track
    final project = ref.read(currentProjectProvider);
    String? trackId;

    if (state.selectedTrackId != null) {
      trackId = state.selectedTrackId;
    } else if (state.selectedClipId != null && project != null) {
      for (final track in project.tracks) {
        for (final clip in track.clips) {
          if (clip.id == state.selectedClipId) {
            trackId = track.id;
            break;
          }
        }
        if (trackId != null) break;
      }
    }

    if (trackId == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.graphic_eq, size: 48, color: AppTheme.textDisabled),
              const SizedBox(height: 16),
              Text(
                'Select a Track',
                style: context.textTheme.titleSmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                'Select a clip or track\nto adjust audio EQ',
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }
    return AudioEqPanel(trackId: trackId);
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
