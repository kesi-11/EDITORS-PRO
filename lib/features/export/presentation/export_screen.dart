import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/constants/app_constants.dart';
import '../../projects/providers/project_provider.dart';
import '../providers/export_provider.dart';

/// Export screen — Configure and export the final video.
///
/// Phase 3 redesign with:
/// - Resolution presets (720p, 1080p, 4K, Social Vertical, Social Square)
/// - Codec selection (H.264, H.265, VP9)
/// - Format selection (MP4, WebM, MOV)
/// - Real-time progress bar with stage indicators
/// - Estimated time remaining
/// - Share button after export
/// - Cancel button during export
class ExportScreen extends ConsumerStatefulWidget {
  final String projectId;

  const ExportScreen({super.key, required this.projectId});

  @override
  ConsumerState<ExportScreen> createState() => _ExportScreenState();
}

class _ExportScreenState extends ConsumerState<ExportScreen>
    with TickerProviderStateMixin {
  String _selectedPreset = '1080p';
  String _selectedCodec = 'H.264';
  String _selectedFormat = 'MP4';
  late AnimationController _pulseController;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final project = ref.watch(currentProjectProvider);
    final exportState = ref.watch(exportProvider);
    final presets = ref.watch(exportPresetsProvider);

    return PopScope(
      canPop: !exportState.isExporting,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop && exportState.isExporting) {
          _showCancelDialog(context);
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Export Video'),
          leading: IconButton(
            onPressed: exportState.isExporting
                ? null
                : () => context.go('/editor/${widget.projectId}'),
            icon: const Icon(Icons.arrow_back),
          ),
          actions: [
            if (exportState.isExporting)
              Padding(
                padding: const EdgeInsets.only(right: 16),
                child: Center(
                  child: Text(
                    exportState.progressText,
                    style: context.textTheme.titleSmall?.copyWith(
                      color: AppTheme.accent,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(20),
          children: [
            // ── Preview summary card ──────────────────────────────
            _buildPreviewCard(context, project),
            const SizedBox(height: 24),

            // ── Resolution presets ────────────────────────────────
            if (!exportState.isExporting && !exportState.isComplete) ...[
              Text('Resolution', style: context.textTheme.titleMedium),
              const SizedBox(height: 12),
              _buildPresetSelector(presets),
              const SizedBox(height: 24),

              // ── Codec selection ─────────────────────────────────
              Text('Codec', style: context.textTheme.titleMedium),
              const SizedBox(height: 12),
              _buildCodecSelector(),
              const SizedBox(height: 24),

              // ── Format selection ────────────────────────────────
              Text('Format', style: context.textTheme.titleMedium),
              const SizedBox(height: 12),
              _buildFormatSelector(),
              const SizedBox(height: 24),

              // ── Export summary ──────────────────────────────────
              _buildExportSummary(context),
              const SizedBox(height: 32),
            ],

            // ── Export progress / status ──────────────────────────
            if (exportState.isExporting)
              _buildExportProgress(context, exportState)
            else if (exportState.isComplete)
              _buildExportComplete(context, exportState)
            else if (exportState.hasError)
              _buildExportError(context, exportState)
            else
              _buildExportButton(context),

            // ── Spacer for bottom ─────────────────────────────────
            const SizedBox(height: 32),
          ],
        ),
      ),
    );
  }

  // ── Preview Card ────────────────────────────────────────────────

  Widget _buildPreviewCard(BuildContext context, dynamic project) {
    return Container(
      height: 160,
      decoration: BoxDecoration(
        color: Colors.black,
        borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
        border: Border.all(color: AppTheme.surfaceVariant, width: 1),
      ),
      child: Stack(
        children: [
          Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.movie_creation_outlined,
                    size: 40, color: AppTheme.textDisabled),
                const SizedBox(height: 8),
                Text(
                  project?.name ?? 'Untitled',
                  style: context.textTheme.titleMedium
                      ?.copyWith(color: Colors.white),
                ),
                if (project != null)
                  Text(
                    '${project.width}x${project.height} @ ${project.fps}fps',
                    style: context.textTheme.bodySmall
                        ?.copyWith(color: AppTheme.textSecondary),
                  ),
              ],
            ),
          ),
          // Resolution badge
          Positioned(
            top: 8,
            right: 8,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: AppTheme.accent.withValues(alpha: 0.9),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                _selectedPreset,
                style: const TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }

  // ── Preset Selector ─────────────────────────────────────────────

  Widget _buildPresetSelector(List<ExportPreset> presets) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: presets.map((preset) {
        final isSelected = _selectedPreset == preset.name;
        return ChoiceChip(
          label: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(preset.name, style: const TextStyle(fontSize: 12)),
              Text('${preset.width}x${preset.height}',
                  style: TextStyle(
                      fontSize: 9, color: AppTheme.textSecondary)),
            ],
          ),
          selected: isSelected,
          onSelected: (_) => setState(() => _selectedPreset = preset.name),
          selectedColor: AppTheme.accent.withValues(alpha: 0.2),
          side: BorderSide(
            color: isSelected ? AppTheme.accent : AppTheme.surfaceVariant,
          ),
        );
      }).toList(),
    );
  }

  // ── Codec Selector ──────────────────────────────────────────────

  Widget _buildCodecSelector() {
    return SegmentedButton<String>(
      segments: const [
        ButtonSegment(
          value: 'H.264',
          label: Text('H.264'),
          tooltip: 'Best compatibility',
        ),
        ButtonSegment(
          value: 'H.265',
          label: Text('H.265'),
          tooltip: 'Better compression',
        ),
        ButtonSegment(
          value: 'VP9',
          label: Text('VP9'),
          tooltip: 'Web optimized',
        ),
      ],
      selected: {_selectedCodec},
      onSelectionChanged: (sel) => setState(() => _selectedCodec = sel.first),
    );
  }

  // ── Format Selector ─────────────────────────────────────────────

  Widget _buildFormatSelector() {
    return SegmentedButton<String>(
      segments: const [
        ButtonSegment(value: 'MP4', label: Text('MP4')),
        ButtonSegment(value: 'WebM', label: Text('WebM')),
        ButtonSegment(value: 'MOV', label: Text('MOV')),
      ],
      selected: {_selectedFormat},
      onSelectionChanged: (sel) => setState(() => _selectedFormat = sel.first),
    );
  }

  // ── Export Summary ───────────────────────────────────────────────

  Widget _buildExportSummary(BuildContext context) {
    final presetMap = {
      '720p': (1280, 720, 5000),
      '1080p': (1920, 1080, 10000),
      '4K': (3840, 2160, 40000),
      'Social Vertical': (1080, 1920, 8000),
      'Social Square': (1080, 1080, 6000),
    };
    final (w, h, bitrate) = presetMap[_selectedPreset] ?? (1920, 1080, 10000);
    final estimatedSizeMb = (bitrate * 60 / 8 / 1024).round();

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
            _SummaryRow(
                label: 'Est. size (1min)', value: '~${estimatedSizeMb}MB'),
          ],
        ),
      ),
    );
  }

  // ── Export Button ────────────────────────────────────────────────

  Widget _buildExportButton(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      height: 52,
      child: ElevatedButton.icon(
        onPressed: _startExport,
        icon: const Icon(Icons.file_download),
        label: const Text('Export Video'),
        style: ElevatedButton.styleFrom(
          backgroundColor: AppTheme.accent,
          foregroundColor: Colors.white,
        ),
      ),
    );
  }

  // ── Export Progress ──────────────────────────────────────────────

  Widget _buildExportProgress(BuildContext context, ExportState state) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Stage indicator
            Row(
              children: [
                _buildStageIndicator('Prepare',
                    _isStageActive('Preparing', state.stageName)),
                _buildStageConnector(
                    _isStageCompleted('Preparing', state.stageName)),
                _buildStageIndicator('Encode',
                    _isStageActive('Encoding', state.stageName)),
                _buildStageConnector(
                    _isStageCompleted('Encoding', state.stageName)),
                _buildStageIndicator('Finalize',
                    _isStageActive('Finalizing', state.stageName)),
              ],
            ),
            const SizedBox(height: 20),

            // Progress bar
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: state.progress,
                backgroundColor: AppTheme.surfaceVariant,
                valueColor: AlwaysStoppedAnimation<Color>(AppTheme.accent),
                minHeight: 10,
              ),
            ),
            const SizedBox(height: 12),

            // Progress text
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  state.progressText,
                  style: context.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                if (state.estimatedTimeText.isNotEmpty)
                  Text(
                    state.estimatedTimeText,
                    style: context.textTheme.bodySmall?.copyWith(
                      color: AppTheme.textSecondary,
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 4),

            // Frame progress
            if (state.frameProgressText.isNotEmpty)
              Text(
                state.frameProgressText,
                style: context.textTheme.bodySmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),

            const SizedBox(height: 16),

            // Stage name with pulse animation
            Row(
              children: [
                FadeTransition(
                  opacity: _pulseController,
                  child: Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: AppTheme.accent,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  '${state.stageName}...',
                  style: context.textTheme.bodyMedium?.copyWith(
                    color: AppTheme.accent,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 20),

            // Warning
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.orange.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                    color: Colors.orange.withValues(alpha: 0.3)),
              ),
              child: Row(
                children: [
                  Icon(Icons.info_outline,
                      size: 18, color: Colors.orange.shade300),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Do not close the app during export. The foreground service will keep the export running if minimized.',
                      style: context.textTheme.bodySmall?.copyWith(
                        color: Colors.orange.shade300,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),

            // Cancel button
            Center(
              child: OutlinedButton.icon(
                onPressed: () => _showCancelDialog(context),
                icon: const Icon(Icons.cancel, size: 18),
                label: const Text('Cancel Export'),
                style: OutlinedButton.styleFrom(
                  foregroundColor: Colors.red.shade300,
                  side: BorderSide(color: Colors.red.shade300!),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStageIndicator(String label, bool active) {
    return Expanded(
      child: Column(
        children: [
          Container(
            width: 28,
            height: 28,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: active ? AppTheme.accent : AppTheme.surfaceVariant,
            ),
            child: active
                ? const Icon(Icons.check, size: 16, color: Colors.white)
                : null,
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 10,
              color: active ? AppTheme.accent : AppTheme.textDisabled,
              fontWeight: active ? FontWeight.bold : FontWeight.normal,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStageConnector(bool completed) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Container(
        width: 24,
        height: 2,
        color: completed ? AppTheme.accent : AppTheme.surfaceVariant,
      ),
    );
  }

  bool _isStageActive(String stage, String currentStage) {
    final order = ['Preparing', 'Encoding', 'Finalizing', 'Complete'];
    final currentIndex = order.indexOf(currentStage);
    final stageIndex = order.indexOf(stage);
    return stageIndex <= currentIndex;
  }

  bool _isStageCompleted(String stage, String currentStage) {
    final order = ['Preparing', 'Encoding', 'Finalizing', 'Complete'];
    final currentIndex = order.indexOf(currentStage);
    final stageIndex = order.indexOf(stage);
    return stageIndex < currentIndex;
  }

  // ── Export Complete ──────────────────────────────────────────────

  Widget _buildExportComplete(BuildContext context, ExportState state) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          children: [
            const Icon(Icons.check_circle, size: 64, color: Colors.green),
            const SizedBox(height: 12),
            Text('Export Complete!',
                style: context.textTheme.titleLarge
                    ?.copyWith(color: Colors.green)),
            const SizedBox(height: 8),
            if (state.fileSizeHuman != null)
              Text(
                'File size: ${state.fileSizeHuman}',
                style: context.textTheme.bodyMedium
                    ?.copyWith(color: AppTheme.textSecondary),
              ),
            const SizedBox(height: 24),

            // Share button
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: () =>
                    ref.read(exportProvider.notifier).shareExportedFile(),
                icon: const Icon(Icons.share),
                label: const Text('Share Video'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.accent,
                  foregroundColor: Colors.white,
                ),
              ),
            ),
            const SizedBox(height: 12),

            // Export again button
            SizedBox(
              width: double.infinity,
              height: 48,
              child: OutlinedButton.icon(
                onPressed: () =>
                    ref.read(exportProvider.notifier).reset(),
                icon: const Icon(Icons.refresh),
                label: const Text('Export Again'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ── Export Error ─────────────────────────────────────────────────

  Widget _buildExportError(BuildContext context, ExportState state) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          children: [
            const Icon(Icons.error_outline, size: 64, color: Colors.red),
            const SizedBox(height: 12),
            Text('Export Failed',
                style: context.textTheme.titleLarge
                    ?.copyWith(color: Colors.red)),
            const SizedBox(height: 8),
            Text(
              state.error ?? 'Unknown error',
              style: context.textTheme.bodyMedium
                  ?.copyWith(color: AppTheme.textSecondary),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),

            // Try again button
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: () => ref.read(exportProvider.notifier).reset(),
                icon: const Icon(Icons.refresh),
                label: const Text('Try Again'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.accent,
                  foregroundColor: Colors.white,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ── Helpers ──────────────────────────────────────────────────────

  void _startExport() {
    ref.read(exportProvider.notifier).startExport(
          preset: _selectedPreset,
          codec: _selectedCodec,
          format: _selectedFormat,
        );
  }

  Future<void> _showCancelDialog(BuildContext context) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Cancel Export?'),
        content: const Text(
            'The current export will be stopped and the partial file will be deleted.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Continue Export'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('Cancel Export'),
          ),
        ],
      ),
    );

    if (confirm == true) {
      ref.read(exportProvider.notifier).cancelExport();
    }
  }
}

// ── Summary Row Widget ─────────────────────────────────────────────

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
          Text(value,
              style: context.textTheme.bodyMedium
                  ?.copyWith(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
