import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../projects/providers/project_provider.dart';
import '../../editor/providers/editor_provider.dart';

/// Export screen - Configure and export the final video
class ExportScreen extends ConsumerStatefulWidget {
  final String projectId;

  const ExportScreen({super.key, required this.projectId});

  @override
  ConsumerState<ExportScreen> createState() => _ExportScreenState();
}

class _ExportScreenState extends ConsumerState<ExportScreen> {
  String _selectedPreset = '1080p';
  String _selectedCodec = 'H.264';
  String _selectedFormat = 'MP4';
  bool _isExporting = false;
  double _exportProgress = 0;

  final _presets = {
    '720p': (1280, 720, 5000),
    '1080p': (1920, 1080, 10000),
    '4K': (3840, 2160, 40000),
    'Social Vertical': (1080, 1920, 8000),
    'Social Square': (1080, 1080, 6000),
  };

  @override
  Widget build(BuildContext context) {
    final project = ref.watch(currentProjectProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Export Video'),
        leading: IconButton(
          onPressed: () => context.go('/editor/${widget.projectId}'),
          icon: const Icon(Icons.arrow_back),
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          // Preview
          Container(
            height: 200,
            decoration: BoxDecoration(
              color: Colors.black,
              borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
            ),
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.movie, size: 48, color: AppTheme.textDisabled),
                  const SizedBox(height: 12),
                  Text(
                    project?.name ?? 'Untitled',
                    style: context.textTheme.titleMedium,
                  ),
                  if (project != null)
                    Text(
                      '${project.width}x${project.height} @ ${project.fps}fps',
                      style: context.textTheme.bodySmall,
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // Resolution presets
          Text('Resolution', style: context.textTheme.titleMedium),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: _presets.keys.map((preset) {
              final isSelected = _selectedPreset == preset;
              final (w, h, _) = _presets[preset]!;
              return ChoiceChip(
                label: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(preset, style: const TextStyle(fontSize: 11)),
                    Text('$w x $h', style: const TextStyle(fontSize: 9)),
                  ],
                ),
                selected: isSelected,
                onSelected: (_) => setState(() => _selectedPreset = preset),
              );
            }).toList(),
          ),
          const SizedBox(height: 24),

          // Codec
          Text('Codec', style: context.textTheme.titleMedium),
          const SizedBox(height: 12),
          SegmentedButton<String>(
            segments: const [
              ButtonSegment(value: 'H.264', label: Text('H.264'), tooltip: 'Best compatibility'),
              ButtonSegment(value: 'H.265', label: Text('H.265'), tooltip: 'Better compression'),
              ButtonSegment(value: 'VP9', label: Text('VP9'), tooltip: 'Web optimized'),
            ],
            selected: {_selectedCodec},
            onSelectionChanged: (selection) => setState(() => _selectedCodec = selection.first),
          ),
          const SizedBox(height: 24),

          // Format
          Text('Format', style: context.textTheme.titleMedium),
          const SizedBox(height: 12),
          SegmentedButton<String>(
            segments: const [
              ButtonSegment(value: 'MP4', label: Text('MP4')),
              ButtonSegment(value: 'WebM', label: Text('WebM')),
              ButtonSegment(value: 'MOV', label: Text('MOV')),
            ],
            selected: {_selectedFormat},
            onSelectionChanged: (selection) => setState(() => _selectedFormat = selection.first),
          ),
          const SizedBox(height: 24),

          // Export summary
          _buildExportSummary(context),
          const SizedBox(height: 32),

          // Export button or progress
          if (_isExporting)
            _buildExportProgress(context)
          else
            SizedBox(
              width: double.infinity,
              height: 52,
              child: ElevatedButton.icon(
                onPressed: () => _startExport(),
                icon: const Icon(Icons.file_download),
                label: const Text('Export Video'),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildExportSummary(BuildContext context) {
    final (w, h, bitrate) = _presets[_selectedPreset]!;
    final estimatedSize = (bitrate * 60 / 8 / 1024).round(); // Rough 1-minute estimate in MB

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Export Summary', style: context.textTheme.titleSmall),
            const SizedBox(height: 12),
            _SummaryRow(label: 'Resolution', value: '$w x $h'),
            _SummaryRow(label: 'Codec', value: _selectedCodec),
            _SummaryRow(label: 'Format', value: _selectedFormat),
            _SummaryRow(label: 'Bitrate', value: '${bitrate}kbps'),
            _SummaryRow(label: 'Est. size (1min)', value: '~${estimatedSize}MB'),
          ],
        ),
      ),
    );
  }

  Widget _buildExportProgress(BuildContext context) {
    return Column(
      children: [
        LinearProgressIndicator(
          value: _exportProgress,
          backgroundColor: AppTheme.surfaceVariant,
          minHeight: 8,
          borderRadius: BorderRadius.circular(4),
        ),
        const SizedBox(height: 12),
        Text(
          '${(_exportProgress * 100).round()}% complete',
          style: context.textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        Text(
          'Rendering video... Do not close the app.',
          style: context.textTheme.bodySmall,
        ),
        const SizedBox(height: 16),
        OutlinedButton(
          onPressed: () {
            setState(() {
              _isExporting = false;
              _exportProgress = 0;
            });
          },
          child: const Text('Cancel'),
        ),
      ],
    );
  }

  void _startExport() {
    setState(() {
      _isExporting = true;
      _exportProgress = 0;
    });

    // Simulate export progress
    // In production: call engine.export_video() and listen to progress stream
    Future.delayed(const Duration(milliseconds: 200), () {
      if (!_isExporting) return;
      setState(() => _exportProgress = 0.1);
      Future.delayed(const Duration(milliseconds: 300), () {
        if (!_isExporting) return;
        setState(() => _exportProgress = 0.3);
        Future.delayed(const Duration(milliseconds: 400), () {
          if (!_isExporting) return;
          setState(() => _exportProgress = 0.6);
          Future.delayed(const Duration(milliseconds: 300), () {
            if (!_isExporting) return;
            setState(() => _exportProgress = 0.85);
            Future.delayed(const Duration(milliseconds: 200), () {
              if (!_isExporting) return;
              setState(() {
                _exportProgress = 1.0;
                _isExporting = false;
              });
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Export complete!')),
              );
            });
          });
        });
      });
    });
  }
}

class _SummaryRow extends StatelessWidget {
  final String label;
  final String value;

  const _SummaryRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: context.textTheme.bodySmall),
          Text(value, style: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
