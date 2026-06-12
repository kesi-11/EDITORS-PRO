/// Settings screen for EDITORS-PRO.
///
/// Provides user-configurable preferences for the editor including
/// default project settings, storage management, and performance tuning.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../../core/services/database_provider.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  bool _autoSaveEnabled = true;
  int _autoSaveIntervalMinutes = 2;
  String _defaultResolution = '1080p';
  double _defaultFps = 30.0;
  bool _proxyEnabled = false;
  String _proxyQuality = '480p';
  bool _hardwareDecoding = true;
  int _cacheSizeMb = 500;

  @override
  void initState() {
    super.initState();
    _loadPreferences();
  }

  Future<void> _loadPreferences() async {
    final db = ref.read(databaseProvider);
    final autoSave = await db.getPreference('auto_save_enabled');
    final autoSaveInterval = await db.getPreference('auto_save_interval');
    final defaultRes = await db.getPreference('default_resolution');
    final defaultFps = await db.getPreference('default_fps');
    final proxyEnabled = await db.getPreference('proxy_enabled');
    final proxyQuality = await db.getPreference('proxy_quality');
    final hwDecode = await db.getPreference('hardware_decoding');
    final cacheSize = await db.getPreference('cache_size_mb');

    if (mounted) {
      setState(() {
        _autoSaveEnabled = autoSave != 'false';
        _autoSaveIntervalMinutes = int.tryParse(autoSaveInterval ?? '') ?? 2;
        _defaultResolution = defaultRes ?? '1080p';
        _defaultFps = double.tryParse(defaultFps ?? '') ?? 30.0;
        _proxyEnabled = proxyEnabled == 'true';
        _proxyQuality = proxyQuality ?? '480p';
        _hardwareDecoding = hwDecode != 'false';
        _cacheSizeMb = int.tryParse(cacheSize ?? '') ?? 500;
      });
    }
  }

  Future<void> _setPreference(String key, String value) async {
    final db = ref.read(databaseProvider);
    await db.setPreference(key, value);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // ─── Project Defaults ──────────────────────────────
          _SectionHeader(title: 'Project Defaults'),
          _SettingsCard(
            children: [
              _SettingsDropdown(
                label: 'Default Resolution',
                value: _defaultResolution,
                options: const ['720p', '1080p', '4K'],
                onChanged: (v) {
                  setState(() => _defaultResolution = v);
                  _setPreference('default_resolution', v);
                },
              ),
              const Divider(height: 1),
              _SettingsDropdown(
                label: 'Default Frame Rate',
                value: _defaultFps == 24.0
                    ? '24 fps'
                    : _defaultFps == 25.0
                        ? '25 fps'
                        : _defaultFps == 30.0
                            ? '30 fps'
                            : _defaultFps == 60.0
                                ? '60 fps'
                                : '30 fps',
                options: const ['24 fps', '25 fps', '30 fps', '60 fps'],
                onChanged: (v) {
                  final fps = double.parse(v.replaceAll(' fps', ''));
                  setState(() => _defaultFps = fps);
                  _setPreference('default_fps', fps.toString());
                },
              ),
            ],
          ),

          const SizedBox(height: 24),

          // ─── Editor Behavior ───────────────────────────────
          _SectionHeader(title: 'Editor Behavior'),
          _SettingsCard(
            children: [
              _SettingsSwitch(
                label: 'Auto-Save',
                subtitle: 'Automatically save project every $_autoSaveIntervalMinutes min',
                value: _autoSaveEnabled,
                onChanged: (v) {
                  setState(() => _autoSaveEnabled = v);
                  _setPreference('auto_save_enabled', v.toString());
                },
              ),
              if (_autoSaveEnabled) ...[
                const Divider(height: 1),
                _SettingsSlider(
                  label: 'Auto-Save Interval',
                  value: _autoSaveIntervalMinutes.toDouble(),
                  min: 1,
                  max: 10,
                  divisions: 9,
                  unit: 'min',
                  onChanged: (v) {
                    setState(() => _autoSaveIntervalMinutes = v.round());
                    _setPreference('auto_save_interval', v.round().toString());
                  },
                ),
              ],
            ],
          ),

          const SizedBox(height: 24),

          // ─── Performance ───────────────────────────────────
          _SectionHeader(title: 'Performance'),
          _SettingsCard(
            children: [
              _SettingsSwitch(
                label: 'Hardware-Accelerated Decoding',
                subtitle: 'Use MediaCodec for faster video decode',
                value: _hardwareDecoding,
                onChanged: (v) {
                  setState(() => _hardwareDecoding = v);
                  _setPreference('hardware_decoding', v.toString());
                },
              ),
              const Divider(height: 1),
              _SettingsSwitch(
                label: 'Proxy Editing',
                subtitle: 'Use lower-resolution proxies for smoother editing',
                value: _proxyEnabled,
                onChanged: (v) {
                  setState(() => _proxyEnabled = v);
                  _setPreference('proxy_enabled', v.toString());
                },
              ),
              if (_proxyEnabled) ...[
                const Divider(height: 1),
                _SettingsDropdown(
                  label: 'Proxy Quality',
                  value: _proxyQuality,
                  options: const ['360p', '480p', '720p'],
                  onChanged: (v) {
                    setState(() => _proxyQuality = v);
                    _setPreference('proxy_quality', v);
                  },
                ),
              ],
              const Divider(height: 1),
              _SettingsSlider(
                label: 'Preview Cache Size',
                value: _cacheSizeMb.toDouble(),
                min: 100,
                max: 2000,
                divisions: 19,
                unit: 'MB',
                onChanged: (v) {
                  setState(() => _cacheSizeMb = v.round());
                  _setPreference('cache_size_mb', v.round().toString());
                },
              ),
            ],
          ),

          const SizedBox(height: 24),

          // ─── Storage ───────────────────────────────────────
          _SectionHeader(title: 'Storage'),
          _SettingsCard(
            children: [
              ListTile(
                leading: const Icon(Icons.folder_outlined, color: AppTheme.primary),
                title: Text('Clear Preview Cache', style: context.textTheme.bodyMedium),
                subtitle: Text('Free up disk space used by cached frames', style: context.textTheme.bodySmall),
                trailing: Icon(Icons.chevron_right, color: AppTheme.textDisabled),
                onTap: () => _showClearCacheDialog(context),
              ),
              const Divider(height: 1),
              ListTile(
                leading: const Icon(Icons.delete_sweep_outlined, color: AppTheme.error),
                title: Text('Delete All Projects', style: context.textTheme.bodyMedium),
                subtitle: Text('Permanently remove all projects and assets', style: context.textTheme.bodySmall),
                trailing: Icon(Icons.chevron_right, color: AppTheme.textDisabled),
                onTap: () => _showDeleteAllDialog(context),
              ),
            ],
          ),

          const SizedBox(height: 32),

          // ─── About ─────────────────────────────────────────
          _SectionHeader(title: 'About'),
          _SettingsCard(
            children: [
              ListTile(
                leading: const Icon(Icons.info_outline, color: AppTheme.textSecondary),
                title: Text('Version', style: context.textTheme.bodyMedium),
                trailing: Text(
                  '${AppConstants.appVersion}',
                  style: context.textTheme.bodySmall?.copyWith(color: AppTheme.textSecondary),
                ),
              ),
              const Divider(height: 1),
              ListTile(
                leading: const Icon(Icons.code, color: AppTheme.textSecondary),
                title: Text('Architecture', style: context.textTheme.bodyMedium),
                trailing: Text(
                  'Flutter + Rust',
                  style: context.textTheme.bodySmall?.copyWith(color: AppTheme.textSecondary),
                ),
              ),
            ],
          ),

          const SizedBox(height: 48),
        ],
      ),
    );
  }

  void _showClearCacheDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Clear Preview Cache?'),
        content: const Text(
          'This will delete all cached preview frames and thumbnails. '
          'Your projects will not be affected.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(ctx);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Cache cleared')),
              );
            },
            child: const Text('Clear'),
          ),
        ],
      ),
    );
  }

  void _showDeleteAllDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete All Projects?'),
        content: const Text(
          'This will permanently delete ALL projects and their associated media. '
          'This action cannot be undone.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: AppTheme.error),
            onPressed: () {
              Navigator.pop(ctx);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('All projects deleted')),
              );
            },
            child: const Text('Delete All'),
          ),
        ],
      ),
    );
  }
}

// ─── Reusable Settings Widgets ────────────────────────────────────

class _SectionHeader extends StatelessWidget {
  final String title;
  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8, left: 4),
      child: Text(
        title.toUpperCase(),
        style: context.textTheme.labelMedium?.copyWith(
          color: AppTheme.primaryLight,
          letterSpacing: 1.2,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}

class _SettingsCard extends StatelessWidget {
  final List<Widget> children;
  const _SettingsCard({required this.children});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.zero,
      child: Column(mainAxisSize: MainAxisSize.min, children: children),
    );
  }
}

class _SettingsSwitch extends StatelessWidget {
  final String label;
  final String? subtitle;
  final bool value;
  final ValueChanged<bool> onChanged;

  const _SettingsSwitch({
    required this.label,
    this.subtitle,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return SwitchListTile(
      title: Text(label, style: context.textTheme.bodyMedium),
      subtitle: subtitle != null
          ? Text(subtitle!, style: context.textTheme.bodySmall)
          : null,
      value: value,
      onChanged: onChanged,
      activeColor: AppTheme.primary,
    );
  }
}

class _SettingsDropdown extends StatelessWidget {
  final String label;
  final String value;
  final List<String> options;
  final ValueChanged<String> onChanged;

  const _SettingsDropdown({
    required this.label,
    required this.value,
    required this.options,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(label, style: context.textTheme.bodyMedium),
      trailing: DropdownButton<String>(
        value: value,
        underline: const SizedBox.shrink(),
        items: options
            .map((o) => DropdownMenuItem(value: o, child: Text(o)))
            .toList(),
        onChanged: (v) {
          if (v != null) onChanged(v);
        },
      ),
    );
  }
}

class _SettingsSlider extends StatelessWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final int divisions;
  final String unit;
  final ValueChanged<double> onChanged;

  const _SettingsSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.divisions,
    required this.unit,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
              Text(label, style: context.textTheme.bodyMedium),
              Text(
                '${value.round()} $unit',
                style: context.textTheme.bodySmall?.copyWith(
                  color: AppTheme.primaryLight,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
          Slider(
            value: value,
            min: min,
            max: max,
            divisions: divisions,
            activeColor: AppTheme.primary,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}
