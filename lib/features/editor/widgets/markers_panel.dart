import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../providers/editor_provider.dart';

// ─── Marker Colors ────────────────────────────────────────────────────

enum MarkerColor { red, yellow, green, blue, purple, orange, white }

extension MarkerColorUI on MarkerColor {
  Color get color {
    switch (this) {
      case MarkerColor.red:
        return const Color(0xFFFF5C5C);
      case MarkerColor.yellow:
        return const Color(0xFFFFB84D);
      case MarkerColor.green:
        return const Color(0xFF00D9A0);
      case MarkerColor.blue:
        return const Color(0xFF4DA6FF);
      case MarkerColor.purple:
        return const Color(0xFF6C5CE7);
      case MarkerColor.orange:
        return const Color(0xFFFF9F43);
      case MarkerColor.white:
        return const Color(0xFFF0F0F8);
    }
  }

  String get label {
    switch (this) {
      case MarkerColor.red:
        return 'Red';
      case MarkerColor.yellow:
        return 'Yellow';
      case MarkerColor.green:
        return 'Green';
      case MarkerColor.blue:
        return 'Blue';
      case MarkerColor.purple:
        return 'Purple';
      case MarkerColor.orange:
        return 'Orange';
      case MarkerColor.white:
        return 'White';
    }
  }
}

// ─── Marker Model ─────────────────────────────────────────────────────

class TimelineMarker {
  final String id;
  final String name;
  final int timeMs;
  final MarkerColor color;
  final String? note;
  final DateTime createdAt;

  const TimelineMarker({
    required this.id,
    required this.name,
    required this.timeMs,
    this.color = MarkerColor.yellow,
    this.note,
    required this.createdAt,
  });

  String get timeFormatted {
    final d = Duration(milliseconds: timeMs);
    final h = d.inHours.toString().padLeft(2, '0');
    final m = (d.inMinutes % 60).toString().padLeft(2, '0');
    final s = (d.inSeconds % 60).toString().padLeft(2, '0');
    final f = ((timeMs % 1000) ~/ 33).toString().padLeft(2, '0');
    return '$h:$m:$s:$f';
  }

  TimelineMarker copyWith({
    String? id,
    String? name,
    int? timeMs,
    MarkerColor? color,
    String? note,
    DateTime? createdAt,
  }) {
    return TimelineMarker(
      id: id ?? this.id,
      name: name ?? this.name,
      timeMs: timeMs ?? this.timeMs,
      color: color ?? this.color,
      note: note ?? this.note,
      createdAt: createdAt ?? this.createdAt,
    );
  }
}

// ─── Markers Provider ─────────────────────────────────────────────────

final markersProvider =
    StateNotifierProvider<MarkersNotifier, List<TimelineMarker>>((ref) {
  return MarkersNotifier();
});

class MarkersNotifier extends StateNotifier<List<TimelineMarker>> {
  MarkersNotifier() : super([]);

  void addMarker(int timeMs,
      {String name = 'Marker',
      MarkerColor color = MarkerColor.yellow,
      String? note}) {
    state = [
      ...state,
      TimelineMarker(
        id: DateTime.now().millisecondsSinceEpoch.toString(),
        name: name,
        timeMs: timeMs,
        color: color,
        note: note,
        createdAt: DateTime.now(),
      ),
    ];
  }

  void removeMarker(String id) {
    state = state.where((m) => m.id != id).toList();
  }

  void updateMarker(String id,
      {String? name, MarkerColor? color, String? note}) {
    state = state.map((m) {
      if (m.id == id) {
        return TimelineMarker(
          id: m.id,
          name: name ?? m.name,
          timeMs: m.timeMs,
          color: color ?? m.color,
          note: note ?? m.note,
          createdAt: m.createdAt,
        );
      }
      return m;
    }).toList();
  }

  void clearAll() {
    state = [];
  }

  TimelineMarker? getMarkerAtTime(int timeMs, {int toleranceMs = 500}) {
    for (final m in state) {
      if ((m.timeMs - timeMs).abs() <= toleranceMs) return m;
    }
    return null;
  }

  List<TimelineMarker> getMarkersInRange(int startMs, int endMs) {
    return state
        .where((m) => m.timeMs >= startMs && m.timeMs <= endMs)
        .toList();
  }
}

// ─── Playhead time provider (reads from editor state) ─────────────────

/// Provided by the editor screen; defaults to zero when not overridden.
final playheadTimeProvider = Provider<int>((ref) {
  return ref.watch(editorProvider.select((s) => s.currentTimeMs));
});

// ─── Markers Panel ────────────────────────────────────────────────────

class MarkersPanel extends ConsumerStatefulWidget {
  const MarkersPanel({super.key});

  @override
  ConsumerState<MarkersPanel> createState() => _MarkersPanelState();
}

class _MarkersPanelState extends ConsumerState<MarkersPanel> {
  MarkerColor? _filterColor;
  bool _sortAscending = true;
  String _editingMarkerId = '';
  final _nameController = TextEditingController();
  final _noteController = TextEditingController();

  // ── Filtered + sorted markers ─────────────────────────────────────

  List<TimelineMarker> get _displayMarkers {
    var markers = ref.read(markersProvider);
    if (_filterColor != null) {
      markers = markers.where((m) => m.color == _filterColor).toList();
    }
    markers = List.of(markers);
    markers.sort((a, b) =>
        _sortAscending ? a.timeMs.compareTo(b.timeMs) : b.timeMs.compareTo(a.timeMs));
    return markers;
  }

  // ── Export helpers ────────────────────────────────────────────────

  String _toCsv(List<TimelineMarker> markers) {
    final buf = StringBuffer('Name,Timecode,Color,Note\n');
    for (final m in markers) {
      final note = (m.note ?? '').replaceAll('"', '""');
      buf.writeln('"${m.name}","${m.timeFormatted}",${m.color.label},"$note"');
    }
    return buf.toString();
  }

  String _toSrt(List<TimelineMarker> markers) {
    final buf = StringBuffer();
    for (var i = 0; i < markers.length; i++) {
      final m = markers[i];
      final start = _msToSrt(m.timeMs);
      // Markers last 1 second in SRT output
      final end = _msToSrt(m.timeMs + 1000);
      buf.writeln('${i + 1}');
      buf.writeln('$start --> $end');
      buf.writeln('${m.name}${m.note != null ? '\n${m.note}' : ''}');
      buf.writeln();
    }
    return buf.toString();
  }

  String _msToSrt(int ms) {
    final h = (ms ~/ 3600000).toString().padLeft(2, '0');
    final m = ((ms % 3600000) ~/ 60000).toString().padLeft(2, '0');
    final s = ((ms % 60000) ~/ 1000).toString().padLeft(2, '0');
    final mil = (ms % 1000).toString().padLeft(3, '0');
    return '$h:$m:$s,$mil';
  }

  // ── Build ─────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final markers = ref.watch(markersProvider);
    final display = _displayMarkers;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header ──────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(
              horizontal: AppTheme.spacing12, vertical: AppTheme.spacing8),
          child: Row(
            children: [
              Text('Markers', style: context.textTheme.titleMedium),
              const Spacer(),
              Text(
                '${markers.length}',
                style: context.textTheme.labelMedium?.copyWith(
                  color: AppTheme.primary,
                ),
              ),
            ],
          ),
        ),

        // ── Action row ──────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacing12),
          child: Row(
            children: [
              _ActionChip(
                icon: Icons.add_location_alt,
                label: 'Add at Playhead',
                onTap: () {
                  final timeMs = ref.read(playheadTimeProvider);
                  ref
                      .read(markersProvider.notifier)
                      .addMarker(timeMs, color: MarkerColor.yellow);
                },
              ),
              const SizedBox(width: 6),
              _ActionChip(
                icon: Icons.delete_sweep_outlined,
                label: 'Clear All',
                danger: true,
                onTap: markers.isEmpty
                    ? null
                    : () {
                        ref.read(markersProvider.notifier).clearAll();
                      },
              ),
              const Spacer(),
              _ActionChip(
                icon: Icons.file_download_outlined,
                label: 'Export',
                onTap: markers.isEmpty
                    ? null
                    : () => _showExportSheet(context, markers),
              ),
            ],
          ),
        ),
        const SizedBox(height: AppTheme.spacing8),

        // ── Filter + sort row ───────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacing12),
          child: Row(
            children: [
              // Color filter chips
              Expanded(
                child: SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: Row(
                    children: [
                      _ColorFilterChip(
                        color: null,
                        selected: _filterColor == null,
                        onTap: () => setState(() => _filterColor = null),
                      ),
                      ...MarkerColor.values.map(
                        (c) => Padding(
                          padding: const EdgeInsets.only(left: 4),
                          child: _ColorFilterChip(
                            color: c,
                            selected: _filterColor == c,
                            onTap: () =>
                                setState(() => _filterColor = c),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),
              // Sort toggle
              InkWell(
                onTap: () =>
                    setState(() => _sortAscending = !_sortAscending),
                borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(
                    _sortAscending
                        ? Icons.arrow_upward
                        : Icons.arrow_downward,
                    size: 16,
                    color: AppTheme.textSecondary,
                  ),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: AppTheme.spacing4),

        // ── Marker list ─────────────────────────────────────────────
        Expanded(
          child: display.isEmpty
              ? _buildEmptyState()
              : ListView.builder(
                  padding: const EdgeInsets.symmetric(
                      horizontal: AppTheme.spacing12,
                      vertical: AppTheme.spacing4),
                  itemCount: display.length,
                  itemBuilder: (context, index) {
                    return _MarkerTile(
                      marker: display[index],
                      isEditing: _editingMarkerId == display[index].id,
                      nameController: _nameController,
                      noteController: _noteController,
                      onSeek: () => _seekToMarker(display[index]),
                      onEdit: () => _startEditing(display[index]),
                      onSave: () => _saveEditing(display[index].id),
                      onCancel: () => _cancelEditing(),
                      onDelete: () {
                        ref
                            .read(markersProvider.notifier)
                            .removeMarker(display[index].id);
                      },
                      onColorChanged: (c) {
                        ref
                            .read(markersProvider.notifier)
                            .updateMarker(display[index].id, color: c);
                      },
                    );
                  },
                ),
        ),
      ],
    );
  }

  // ── Empty state ────────────────────────────────────────────────────

  Widget _buildEmptyState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(AppTheme.spacing32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.bookmark_border,
                size: 48, color: AppTheme.textDisabled.withOpacity(0.5)),
            const SizedBox(height: AppTheme.spacing12),
            Text(
              _filterColor != null
                  ? 'No ${_filterColor!.label} markers'
                  : 'No markers yet',
              style: context.textTheme.bodyMedium?.copyWith(
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: AppTheme.spacing4),
            Text(
              'Tap "Add at Playhead" to create a marker',
              style: context.textTheme.bodySmall?.copyWith(
                color: AppTheme.textDisabled,
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ── Seek to marker ─────────────────────────────────────────────────

  void _seekToMarker(TimelineMarker marker) {
    // In a full integration this would call the editor provider's seek.
    // For now, show a snackbar as feedback.
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Seek to ${marker.timeFormatted}'),
        duration: const Duration(seconds: 1),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  // ── Inline editing ─────────────────────────────────────────────────

  void _startEditing(TimelineMarker marker) {
    setState(() {
      _editingMarkerId = marker.id;
      _nameController.text = marker.name;
      _noteController.text = marker.note ?? '';
    });
  }

  void _saveEditing(String id) {
    final name = _nameController.text.trim();
    final note = _noteController.text.trim();
    ref.read(markersProvider.notifier).updateMarker(
          id,
          name: name.isEmpty ? 'Marker' : name,
          note: note.isEmpty ? null : note,
        );
    setState(() => _editingMarkerId = '');
    _nameController.clear();
    _noteController.clear();
  }

  void _cancelEditing() {
    setState(() => _editingMarkerId = '');
    _nameController.clear();
    _noteController.clear();
  }

  // ── Export sheet ────────────────────────────────────────────────────

  void _showExportSheet(BuildContext context, List<TimelineMarker> markers) {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(
            top: Radius.circular(AppTheme.radiusXLarge)),
      ),
      builder: (ctx) {
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(AppTheme.spacing20),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Export Markers',
                    style: context.textTheme.titleMedium),
                const SizedBox(height: AppTheme.spacing4),
                Text(
                  'Export ${markers.length} marker${markers.length == 1 ? '' : 's'} as a file.',
                  style: context.textTheme.bodySmall,
                ),
                const SizedBox(height: AppTheme.spacing20),
                Row(
                  children: [
                    Expanded(
                      child: OutlinedButton.icon(
                        onPressed: () {
                          Navigator.pop(ctx);
                          _performExport('csv', markers);
                        },
                        icon: const Icon(Icons.table_chart, size: 18),
                        label: const Text('CSV'),
                      ),
                    ),
                    const SizedBox(width: AppTheme.spacing12),
                    Expanded(
                      child: OutlinedButton.icon(
                        onPressed: () {
                          Navigator.pop(ctx);
                          _performExport('srt', markers);
                        },
                        icon: const Icon(Icons.subtitles, size: 18),
                        label: const Text('SRT'),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  void _performExport(String format, List<TimelineMarker> markers) {
    final content = format == 'csv' ? _toCsv(markers) : _toSrt(markers);
    // In a production app, write to file via file_picker or share_plus.
    // Here we display a success snackbar with the first few lines.
    final preview = content.split('\n').take(5).join('\n');
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Exported as ${format.toUpperCase()}\n$preview'),
        duration: const Duration(seconds: 4),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  @override
  void dispose() {
    _nameController.dispose();
    _noteController.dispose();
    super.dispose();
  }
}

// ─── Action Chip ──────────────────────────────────────────────────────

class _ActionChip extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback? onTap;
  final bool danger;

  const _ActionChip({
    required this.icon,
    required this.label,
    this.onTap,
    this.danger = false,
  });

  @override
  Widget build(BuildContext context) {
    final fg = danger ? AppTheme.error : AppTheme.primary;
    return Material(
      color: fg.withOpacity(0.1),
      borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 14, color: fg),
              const SizedBox(width: 4),
              Text(
                label,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w600,
                  color: onTap != null ? fg : AppTheme.textDisabled,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ─── Color Filter Chip ────────────────────────────────────────────────

class _ColorFilterChip extends StatelessWidget {
  final MarkerColor? color;
  final bool selected;
  final VoidCallback onTap;

  const _ColorFilterChip({
    required this.color,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final radius = BorderRadius.circular(AppTheme.radiusSmall);
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        decoration: BoxDecoration(
          color: selected
              ? (color?.color ?? AppTheme.primary).withOpacity(0.25)
              : AppTheme.surfaceVariant,
          borderRadius: radius,
          border: Border.all(
            color: selected
                ? (color?.color ?? AppTheme.primary)
                : Colors.transparent,
            width: 1.5,
          ),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 10,
              height: 10,
              decoration: BoxDecoration(
                color: color?.color ?? AppTheme.textSecondary,
                shape: BoxShape.circle,
              ),
            ),
            const SizedBox(width: 4),
            Text(
              color?.label ?? 'All',
              style: TextStyle(
                fontSize: 10,
                fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
                color: selected
                    ? (color?.color ?? AppTheme.primary)
                    : AppTheme.textSecondary,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ─── Marker Tile ──────────────────────────────────────────────────────

class _MarkerTile extends StatefulWidget {
  final TimelineMarker marker;
  final bool isEditing;
  final TextEditingController nameController;
  final TextEditingController noteController;
  final VoidCallback onSeek;
  final VoidCallback onEdit;
  final VoidCallback onSave;
  final VoidCallback onCancel;
  final VoidCallback onDelete;
  final ValueChanged<MarkerColor> onColorChanged;

  const _MarkerTile({
    required this.marker,
    required this.isEditing,
    required this.nameController,
    required this.noteController,
    required this.onSeek,
    required this.onEdit,
    required this.onSave,
    required this.onCancel,
    required this.onDelete,
    required this.onColorChanged,
  });

  @override
  State<_MarkerTile> createState() => _MarkerTileState();
}

class _MarkerTileState extends State<_MarkerTile> {
  bool _noteExpanded = false;

  @override
  Widget build(BuildContext context) {
    final m = widget.marker;
    final isEditing = widget.isEditing;

    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Material(
        color: AppTheme.surfaceVariant,
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
        child: InkWell(
          onTap: isEditing ? null : widget.onSeek,
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          child: Padding(
            padding: const EdgeInsets.symmetric(
                horizontal: AppTheme.spacing8, vertical: AppTheme.spacing8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // ── Row 1: color dot, name/timecode, actions ────────
                Row(
                  children: [
                    // Color dot (tappable to cycle color)
                    GestureDetector(
                      onTap: () => _cycleColor(),
                      child: Container(
                        width: 12,
                        height: 12,
                        decoration: BoxDecoration(
                          color: m.color.color,
                          shape: BoxShape.circle,
                          boxShadow: [
                            BoxShadow(
                              color: m.color.color.withOpacity(0.5),
                              blurRadius: 4,
                            ),
                          ],
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),

                    // Name / edit field
                    if (isEditing)
                      Expanded(
                        child: SizedBox(
                          height: 28,
                          child: TextField(
                            controller: widget.nameController,
                            style: const TextStyle(
                              fontSize: 13,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.textPrimary,
                            ),
                            decoration: InputDecoration(
                              filled: true,
                              fillColor: AppTheme.cardColor,
                              contentPadding: const EdgeInsets.symmetric(
                                  horizontal: 8),
                              border: OutlineInputBorder(
                                borderRadius: BorderRadius.circular(
                                    AppTheme.radiusSmall),
                                borderSide: BorderSide.none,
                              ),
                              isDense: true,
                            ),
                            autofocus: true,
                          ),
                        ),
                      )
                    else
                      Expanded(
                        child: Text(
                          m.name,
                          style: const TextStyle(
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),

                    // Timecode
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: AppTheme.cardColor,
                        borderRadius:
                            BorderRadius.circular(AppTheme.radiusSmall),
                      ),
                      child: Text(
                        m.timeFormatted,
                        style: TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.w500,
                          fontFamily: 'monospace',
                          color: AppTheme.textSecondary,
                        ),
                      ),
                    ),
                    const SizedBox(width: 4),

                    // Action buttons
                    if (isEditing) ...[
                      _SmallIconButton(
                        icon: Icons.check,
                        color: AppTheme.success,
                        onTap: widget.onSave,
                      ),
                      _SmallIconButton(
                        icon: Icons.close,
                        color: AppTheme.error,
                        onTap: widget.onCancel,
                      ),
                    ] else ...[
                      _SmallIconButton(
                        icon: Icons.edit_outlined,
                        color: AppTheme.textSecondary,
                        onTap: widget.onEdit,
                      ),
                      _SmallIconButton(
                        icon: Icons.delete_outline,
                        color: AppTheme.error,
                        onTap: widget.onDelete,
                      ),
                    ],
                  ],
                ),

                // ── Row 2: Note ──────────────────────────────────────
                if (isEditing) ...[
                  const SizedBox(height: 6),
                  TextField(
                    controller: widget.noteController,
                    style: const TextStyle(
                      fontSize: 12,
                      color: AppTheme.textPrimary,
                    ),
                    maxLines: 2,
                    decoration: InputDecoration(
                      filled: true,
                      fillColor: AppTheme.cardColor,
                      hintText: 'Add a note...',
                      hintStyle: const TextStyle(
                          color: AppTheme.textDisabled, fontSize: 12),
                      contentPadding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 6),
                      border: OutlineInputBorder(
                        borderRadius:
                            BorderRadius.circular(AppTheme.radiusSmall),
                        borderSide: BorderSide.none,
                      ),
                      isDense: true,
                    ),
                  ),
                ] else if (m.note != null && m.note!.isNotEmpty) ...[
                  const SizedBox(height: 4),
                  GestureDetector(
                    onTap: () => setState(() => _noteExpanded = !_noteExpanded),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Icon(
                          _noteExpanded
                              ? Icons.expand_less
                              : Icons.expand_more,
                          size: 14,
                          color: AppTheme.textDisabled,
                        ),
                        const SizedBox(width: 4),
                        Expanded(
                          child: Text(
                            m.note!,
                            maxLines: _noteExpanded ? null : 1,
                            overflow: _noteExpanded
                                ? TextOverflow.visible
                                : TextOverflow.ellipsis,
                            style: TextStyle(
                              fontSize: 11,
                              color: AppTheme.textSecondary,
                              fontStyle: FontStyle.italic,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _cycleColor() {
    final colors = MarkerColor.values;
    final idx = colors.indexOf(widget.marker.color);
    final next = colors[(idx + 1) % colors.length];
    widget.onColorChanged(next);
  }
}

// ─── Small icon button ────────────────────────────────────────────────

class _SmallIconButton extends StatelessWidget {
  final IconData icon;
  final Color color;
  final VoidCallback onTap;

  const _SmallIconButton({
    required this.icon,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 24,
      height: 24,
      child: IconButton(
        icon: Icon(icon, size: 14),
        color: color,
        padding: EdgeInsets.zero,
        constraints: const BoxConstraints(
          minWidth: 24,
          minHeight: 24,
        ),
        onPressed: onTap,
        splashRadius: 12,
      ),
    );
  }
}
