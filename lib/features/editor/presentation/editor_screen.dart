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
                if (!isNarrowScreen) _buildLeftPanel(context, editorState),

                // Center: Preview viewport
                const Expanded(
                  flex: 3,
                  child: PreviewViewport(),
                ),

                // Right: Inspector / Properties (hidden on narrow screens)
                if (!isNarrowScreen)
                  const Expanded(
                    flex: 1,
                    child: InspectorPanel(),
                  ),
              ],
            ),
          ),

          // Bottom: Timeline
          const TimelinePanel(),
        ],
      ),
    );
  }

  Widget _buildLeftPanel(BuildContext context, EditorState state) {
    return Container(
      width: 240,
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
                        color: AppTheme.audioTrackColor.withValues(alpha: 0.2),
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
    final textPresets = [
      ('Title', 'Large bold text', Icons.title, 72.0),
      ('Subtitle', 'Medium text', Icons.subtitles, 36.0),
      ('Caption', 'Small text with background', Icons.closed_caption, 24.0),
      ('Lower Third', 'Name/title bar', Icons.text_fields, 28.0),
    ];

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: textPresets.length,
      itemBuilder: (context, index) {
        final (name, description, icon, fontSize) = textPresets[index];
        return Card(
          child: ListTile(
            leading: Icon(icon, color: AppTheme.textTrackColor),
            title: Text(name, style: context.textTheme.titleSmall),
            subtitle: Text(description, style: context.textTheme.bodySmall),
            onTap: () => _addTextToTimeline(name, fontSize),
          ),
        );
      },
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
    return ListTile(
      dense: true,
      leading: Container(
        width: 48,
        height: 32,
        decoration: BoxDecoration(
          color: AppTheme.surfaceVariant,
          borderRadius: BorderRadius.circular(4),
        ),
        child: Icon(
          asset.mediaType == MediaType.video
              ? Icons.videocam
              : asset.mediaType == MediaType.audio
                  ? Icons.audiotrack
                  : Icons.image,
          color: AppTheme.textSecondary,
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
        icon: const Icon(Icons.add_circle, size: 18, color: AppTheme.primary),
        onPressed: onAddToTimeline,
        tooltip: 'Add to timeline',
      ),
    );
  }
}

// Note: _EffectCard replaced by EffectCatalog widget in effect_catalog.dart
