import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../../data/models/project_model.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/editor_provider.dart';
import '../widgets/preview_viewport.dart';
import '../widgets/timeline_panel.dart';
import '../widgets/editor_toolbar.dart';
import '../widgets/inspector_panel.dart';

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
        return _MediaAssetItem(asset: asset);
      },
    );
  }

  Widget _buildEffectsPanel(BuildContext context) {
    final effects = [
      ('Brightness', Icons.brightness_6, AppTheme.warning),
      ('Contrast', Icons.contrast, AppTheme.primary),
      ('Saturation', Icons.palette, AppTheme.secondary),
      ('Blur', Icons.blur_on, AppTheme.info),
      ('Sharpen', Icons.deblur, AppTheme.accent),
      ('Grayscale', Icons.gradient, AppTheme.textSecondary),
      ('Sepia', Icons.filter_vintage, AppTheme.warning),
      ('Vignette', Icons.vignette, AppTheme.error),
    ];

    return GridView.builder(
      padding: const EdgeInsets.all(8),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 2,
        childAspectRatio: 1.3,
        crossAxisSpacing: 8,
        mainAxisSpacing: 8,
      ),
      itemCount: effects.length,
      itemBuilder: (context, index) {
        final (name, icon, color) = effects[index];
        return _EffectCard(name: name, icon: icon, color: color);
      },
    );
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
            onTap: () {
              // TODO: Add text clip to timeline
            },
          ),
        );
      },
    );
  }

  Future<void> _importMedia() async {
    // This will use the Rust bridge in production
    // For now, use file_picker to select a file
    try {
      ref.read(editorProvider.notifier).setImporting(true);
      // In production: call engine.import_media(path)
      // For MVP: we'll handle this through the bridge
      ref.read(editorProvider.notifier).setImporting(false);
    } catch (e) {
      ref.read(editorProvider.notifier).setImporting(false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Import failed: $e')),
        );
      }
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

  const _MediaAssetItem({required this.asset});

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
        icon: const Icon(Icons.add_circle_outline, size: 18),
        onPressed: () {
          // TODO: Add to timeline
        },
      ),
    );
  }
}

class _EffectCard extends StatelessWidget {
  final String name;
  final IconData icon;
  final Color color;

  const _EffectCard({
    required this.name,
    required this.icon,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
        onTap: () {
          // TODO: Apply effect
        },
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, color: color, size: 24),
              const SizedBox(height: 8),
              Text(
                name,
                style: context.textTheme.labelMedium,
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
