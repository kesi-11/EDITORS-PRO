import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../providers/editor_provider.dart';

/// Effect catalog panel — displays available filters and presets
/// that can be applied to the selected clip.
class EffectCatalog extends ConsumerStatefulWidget {
  const EffectCatalog({super.key});

  @override
  ConsumerState<EffectCatalog> createState() => _EffectCatalogState();
}

class _EffectCatalogState extends ConsumerState<EffectCatalog>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  List<dynamic> _filterCatalog = [];
  List<dynamic> _presets = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _loadCatalog();
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _loadCatalog() async {
    final notifier = ref.read(editorProvider.notifier);
    final filters = await notifier.getFilterCatalog();
    final presets = await notifier.getFilterPresets();
    if (mounted) {
      setState(() {
        _filterCatalog = filters;
        _presets = presets;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorProvider);
    final hasClip = editorState.selectedClipId != null;

    return Column(
      children: [
        // Tab bar
        TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: 'Filters'),
            Tab(text: 'Presets'),
          ],
          labelColor: AppTheme.primary,
          unselectedLabelColor: AppTheme.textDisabled,
          indicatorColor: AppTheme.primary,
          labelStyle: context.textTheme.labelMedium,
        ),

        // Tab content
        Expanded(
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : TabBarView(
                  controller: _tabController,
                  children: [
                    _buildFilterGrid(hasClip),
                    _buildPresetList(hasClip),
                  ],
                ),
        ),
      ],
    );
  }

  Widget _buildFilterGrid(bool hasClip) {
    if (_filterCatalog.isEmpty) {
      return Center(
        child: Text('No filters available', style: context.textTheme.bodySmall),
      );
    }

    return GridView.builder(
      padding: const EdgeInsets.all(8),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        childAspectRatio: 1.0,
        crossAxisSpacing: 6,
        mainAxisSpacing: 6,
      ),
      itemCount: _filterCatalog.length,
      itemBuilder: (context, index) {
        final filter = _filterCatalog[index];
        final name = filter.name ?? filter['name'] ?? 'Unknown';
        final icon = filter.icon ?? filter['icon'] ?? 'filter';

        return _FilterCard(
          name: name,
          icon: icon,
          enabled: hasClip,
          onTap: () => _addFilter(name),
        );
      },
    );
  }

  Widget _buildPresetList(bool hasClip) {
    if (_presets.isEmpty) {
      return Center(
        child: Text('No presets available', style: context.textTheme.bodySmall),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: _presets.length,
      itemBuilder: (context, index) {
        final preset = _presets[index];
        final id = preset.id ?? preset['id'] ?? '';
        final name = preset.name ?? preset['name'] ?? 'Unknown';
        final description = preset.description ?? preset['description'] ?? '';

        return Card(
          color: AppTheme.surfaceVariant,
          margin: const EdgeInsets.only(bottom: 6),
          child: ListTile(
            dense: true,
            title: Text(name, style: context.textTheme.bodyMedium),
            subtitle: description.isNotEmpty
                ? Text(description, style: context.textTheme.bodySmall)
                : null,
            trailing: Icon(
              Icons.auto_fix_high,
              size: 18,
              color: hasClip ? AppTheme.primary : AppTheme.textDisabled,
            ),
            onTap: hasClip ? () => _applyPreset(id) : null,
          ),
        );
      },
    );
  }

  Future<void> _addFilter(String filterName) async {
    final notifier = ref.read(editorProvider.notifier);
    await notifier.addEffect(filterName);
  }

  Future<void> _applyPreset(String presetId) async {
    final notifier = ref.read(editorProvider.notifier);
    await notifier.applyFilterPreset(presetId);
  }
}

class _FilterCard extends StatelessWidget {
  final String name;
  final String icon;
  final bool enabled;
  final VoidCallback onTap;

  const _FilterCard({
    required this.name,
    required this.icon,
    required this.enabled,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: enabled ? onTap : null,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        decoration: BoxDecoration(
          color: AppTheme.surfaceVariant,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: enabled ? AppTheme.textDisabled.withValues(alpha: 0.2) : Colors.transparent,
            width: 1,
          ),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              _iconData,
              size: 24,
              color: enabled ? AppTheme.textPrimary : AppTheme.textDisabled,
            ),
            const SizedBox(height: 4),
            Text(
              name,
              style: context.textTheme.labelSmall?.copyWith(
                color: enabled ? AppTheme.textPrimary : AppTheme.textDisabled,
              ),
              textAlign: TextAlign.center,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }

  IconData get _iconData {
    switch (icon) {
      case 'brightness': return Icons.brightness_6;
      case 'contrast': return Icons.contrast;
      case 'saturation': return Icons.water_drop;
      case 'hue': return Icons.palette;
      case 'blur': return Icons.blur_on;
      case 'sharpen': return Icons.deblur;
      case 'grayscale': return Icons.gradient;
      case 'sepia': return Icons.filter_vintage;
      case 'invert': return Icons.invert_colors;
      case 'vignette': return Icons.vignette;
      case 'temperature': return Icons.thermostat;
      default: return Icons.auto_fix_high;
    }
  }
}
