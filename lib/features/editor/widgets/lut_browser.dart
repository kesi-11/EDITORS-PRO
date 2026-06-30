import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';

// ─── LUT Model ────────────────────────────────────────────────────────

class LutPreset {
  final String id;
  final String name;
  final String category;
  final List<Color> gradientStops;
  final String description;

  const LutPreset({
    required this.id,
    required this.name,
    required this.category,
    required this.gradientStops,
    this.description = '',
  });
}

// ─── LUT Application State ────────────────────────────────────────────

class LutApplicationState {
  final String? activeLutId;
  final double intensity;
  final bool showBeforeAfter;

  const LutApplicationState({
    this.activeLutId,
    this.intensity = 100.0,
    this.showBeforeAfter = false,
  });

  LutApplicationState copyWith({
    String? activeLutId,
    bool clearLut = false,
    double? intensity,
    bool? showBeforeAfter,
  }) {
    return LutApplicationState(
      activeLutId: clearLut ? null : (activeLutId ?? this.activeLutId),
      intensity: intensity ?? this.intensity,
      showBeforeAfter: showBeforeAfter ?? this.showBeforeAfter,
    );
  }
}

// ─── LUT Application Provider per clip ────────────────────────────────

final _lutStateProviders =
    StateNotifierProvider.family<LutStateNotifier, LutApplicationState, String>(
  (ref, clipId) => LutStateNotifier(),
);

class LutStateNotifier extends StateNotifier<LutApplicationState> {
  LutStateNotifier() : super(const LutApplicationState());

  void applyLut(String lutId) {
    state = state.copyWith(activeLutId: lutId);
  }

  void removeLut() {
    state = state.copyWith(clearLut: true);
  }

  void setIntensity(double value) {
    state = state.copyWith(intensity: value);
  }

  void toggleBeforeAfter(bool value) {
    state = state.copyWith(showBeforeAfter: value);
  }
}

// ─── Built-in LUT Presets ─────────────────────────────────────────────

const _builtInLuts = <LutPreset>[
  LutPreset(
    id: 'rec709_to_srgb',
    name: 'Rec709 → sRGB',
    category: 'Technical',
    description: 'Convert Rec.709 gamut to sRGB color space',
    gradientStops: [
      Color(0xFF1A1A2E),
      Color(0xFF2D3A5C),
      Color(0xFF4A6FA5),
      Color(0xFF7BA1CC),
      Color(0xFFB8D4E8),
      Color(0xFFF0F4F8),
    ],
  ),
  LutPreset(
    id: 'film_warm',
    name: 'Film Look Warm',
    category: 'Film',
    description: 'Warm cinematic tones with lifted blacks',
    gradientStops: [
      Color(0xFF1C1008),
      Color(0xFF3D2814),
      Color(0xFF7A5230),
      Color(0xFFB88A4E),
      Color(0xFFDDB877),
      Color(0xFFF5E6C8),
    ],
  ),
  LutPreset(
    id: 'film_cool',
    name: 'Film Look Cool',
    category: 'Film',
    description: 'Cool cinematic tones with subtle blue shift',
    gradientStops: [
      Color(0xFF0A0E1A),
      Color(0xFF152038),
      Color(0xFF2A3F6E),
      Color(0xFF5A7FAA),
      Color(0xFF8EB4D4),
      Color(0xFFD0E4F2),
    ],
  ),
  LutPreset(
    id: 'teal_orange',
    name: 'Teal & Orange',
    category: 'Film',
    description: 'Hollywood split-tone: teal shadows, orange highlights',
    gradientStops: [
      Color(0xFF0A2E3D),
      Color(0xFF1A5A6E),
      Color(0xFF3A8A8A),
      Color(0xFFC07830),
      Color(0xFFE8944A),
      Color(0xFFFAC88A),
    ],
  ),
  LutPreset(
    id: 'vintage_fade',
    name: 'Vintage Fade',
    category: 'Film',
    description: 'Faded blacks and muted colors for a retro feel',
    gradientStops: [
      Color(0xFF3D3535),
      Color(0xFF6B5B5B),
      Color(0xFF9E8E82),
      Color(0xFFC4B8A8),
      Color(0xFFDED4C8),
      Color(0xFFF2ECE6),
    ],
  ),
  LutPreset(
    id: 'high_contrast_bw',
    name: 'High Contrast BW',
    category: 'Stylize',
    description: 'Punchy black and white with deep blacks',
    gradientStops: [
      Color(0xFF000000),
      Color(0xFF1A1A1A),
      Color(0xFF444444),
      Color(0xFF888888),
      Color(0xFFCCCCCC),
      Color(0xFFFFFFFF),
    ],
  ),
  LutPreset(
    id: 'desaturated',
    name: 'Desaturated',
    category: 'Stylize',
    description: 'Muted colors for a subdued, neutral look',
    gradientStops: [
      Color(0xFF1A1A1E),
      Color(0xFF3A3A42),
      Color(0xFF6A6A76),
      Color(0xFF9A9AA6),
      Color(0xFFCACAD6),
      Color(0xFFF0F0F4),
    ],
  ),
  LutPreset(
    id: 'orange_teal',
    name: 'Orange & Teal',
    category: 'Film',
    description: 'Warm skin tones with teal environment tones',
    gradientStops: [
      Color(0xFF0D2B2E),
      Color(0xFF1E4A4E),
      Color(0xFF5A6A3E),
      Color(0xFFC08030),
      Color(0xFFE8A050),
      Color(0xFFF8D0A0),
    ],
  ),
  LutPreset(
    id: 'moody_blue',
    name: 'Moody Blue',
    category: 'Stylize',
    description: 'Deep blue tones for a moody atmosphere',
    gradientStops: [
      Color(0xFF020818),
      Color(0xFF0A1A3A),
      Color(0xFF163060),
      Color(0xFF2A5090),
      Color(0xFF5080C0),
      Color(0xFF90B8E8),
    ],
  ),
  LutPreset(
    id: 'sunset_warm',
    name: 'Sunset Warm',
    category: 'Stylize',
    description: 'Golden hour warmth with soft orange glow',
    gradientStops: [
      Color(0xFF1A0A04),
      Color(0xFF3D1A0A),
      Color(0xFF8A3A10),
      Color(0xFFD06A20),
      Color(0xFFF0A848),
      Color(0xFFF8E0A0),
    ],
  ),
];

// ─── LUT Browser ──────────────────────────────────────────────────────

class LutBrowser extends ConsumerStatefulWidget {
  final String clipId;

  const LutBrowser({super.key, required this.clipId});

  @override
  ConsumerState<LutBrowser> createState() => _LutBrowserState();
}

class _LutBrowserState extends ConsumerState<LutBrowser> {
  String _selectedCategory = 'All';
  final List<LutPreset> _customLuts = [];

  // ── Categories ────────────────────────────────────────────────────

  List<String> get _categories {
    final cats = <String>{'All'};
    for (final l in [..._builtInLuts, ..._customLuts]) {
      cats.add(l.category);
    }
    return cats.toList()..sort();
  }

  List<LutPreset> get _filteredLuts {
    final all = [..._builtInLuts, ..._customLuts];
    if (_selectedCategory == 'All') return all;
    return all.where((l) => l.category == _selectedCategory).toList();
  }

  // ── Build ─────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final lutState = ref.watch(_lutStateProviders(widget.clipId));
    final luts = _filteredLuts;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header ──────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(
              horizontal: AppTheme.spacing12, vertical: AppTheme.spacing8),
          child: Row(
            children: [
              Text('LUT Browser', style: context.textTheme.titleMedium),
              const Spacer(),
              if (lutState.activeLutId != null)
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(
                    color: AppTheme.primary.withOpacity(0.15),
                    borderRadius:
                        BorderRadius.circular(AppTheme.radiusSmall),
                  ),
                  child: Text(
                    'LUT Active',
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.primary,
                    ),
                  ),
                ),
            ],
          ),
        ),

        // ── Controls row ────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacing12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Intensity slider
              if (lutState.activeLutId != null) ...[
                Row(
                  children: [
                    Text(
                      'Intensity',
                      style: context.textTheme.labelMedium,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Slider(
                        value: lutState.intensity,
                        min: 0,
                        max: 100,
                        divisions: 100,
                        label: '${lutState.intensity.round()}%',
                        onChanged: (v) {
                          ref
                              .read(_lutStateProviders(widget.clipId).notifier)
                              .setIntensity(v);
                        },
                      ),
                    ),
                    SizedBox(
                      width: 42,
                      child: Text(
                        '${lutState.intensity.round()}%',
                        style: context.textTheme.labelSmall?.copyWith(
                          fontFamily: 'monospace',
                        ),
                        textAlign: TextAlign.right,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 4),

                // Before / After toggle + Remove LUT
                Row(
                  children: [
                    _ToggleChip(
                      label: 'Before / After',
                      icon: Icons.compare,
                      active: lutState.showBeforeAfter,
                      onChanged: (v) {
                        ref
                            .read(_lutStateProviders(widget.clipId).notifier)
                            .toggleBeforeAfter(v);
                      },
                    ),
                    const Spacer(),
                    TextButton.icon(
                      onPressed: () {
                        ref
                            .read(_lutStateProviders(widget.clipId).notifier)
                            .removeLut();
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('LUT removed'),
                            duration: Duration(seconds: 1),
                            behavior: SnackBarBehavior.floating,
                          ),
                        );
                      },
                      icon: const Icon(Icons.remove_circle_outline, size: 16),
                      label: const Text('Remove LUT'),
                      style: TextButton.styleFrom(
                        foregroundColor: AppTheme.error,
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        minimumSize: Size.zero,
                        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                const Divider(height: 1),
                const SizedBox(height: 8),
              ],

              // Import LUT + Category filter
              Row(
                children: [
                  _ActionChip(
                    icon: Icons.file_upload_outlined,
                    label: 'Import .cube',
                    onTap: _importLut,
                  ),
                  const Spacer(),
                  // Category dropdown
                  _CategoryDropdown(
                    categories: _categories,
                    selected: _selectedCategory,
                    onChanged: (c) => setState(() => _selectedCategory = c),
                  ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: AppTheme.spacing8),

        // ── LUT grid ────────────────────────────────────────────────
        Expanded(
          child: luts.isEmpty
              ? _buildEmptyState()
              : GridView.builder(
                  padding: const EdgeInsets.symmetric(
                      horizontal: AppTheme.spacing12),
                  gridDelegate:
                      const SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: 2,
                    childAspectRatio: 0.72,
                    crossAxisSpacing: 8,
                    mainAxisSpacing: 8,
                  ),
                  itemCount: luts.length,
                  itemBuilder: (context, index) {
                    final lut = luts[index];
                    final isActive = lutState.activeLutId == lut.id;
                    return _LutCard(
                      lut: lut,
                      isActive: isActive,
                      intensity: isActive ? lutState.intensity : 100.0,
                      showBeforeAfter:
                          isActive && lutState.showBeforeAfter,
                      onTap: () => _applyLut(lut),
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
            Icon(Icons.palette_outlined,
                size: 48, color: AppTheme.textDisabled.withOpacity(0.5)),
            const SizedBox(height: AppTheme.spacing12),
            Text(
              'No LUTs in this category',
              style: context.textTheme.bodyMedium?.copyWith(
                color: AppTheme.textSecondary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ── Apply LUT ──────────────────────────────────────────────────────

  void _applyLut(LutPreset lut) {
    ref
        .read(_lutStateProviders(widget.clipId).notifier)
        .applyLut(lut.id);
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Applied "${lut.name}"'),
        duration: const Duration(seconds: 1),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  // ── Import LUT ─────────────────────────────────────────────────────

  void _importLut() {
    // In production this would use file_picker to select .cube files,
    // parse the LUT data, and add it to the custom list.
    // Here we simulate by adding a placeholder entry.
    final id = 'custom_${DateTime.now().millisecondsSinceEpoch}';
    setState(() {
      _customLuts.add(LutPreset(
        id: id,
        name: 'Custom LUT ${_customLuts.length + 1}',
        category: 'Custom',
        description: 'Imported .cube LUT',
        gradientStops: [
          const Color(0xFF0D0D12),
          const Color(0xFF20203A),
          const Color(0xFF4040A0),
          const Color(0xFF8080C0),
          const Color(0xFFC0C0E0),
          const Color(0xFFF0F0FF),
        ],
      ));
    });
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('LUT file selected (simulated)'),
        duration: Duration(seconds: 2),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}

// ─── LUT Card ─────────────────────────────────────────────────────────

class _LutCard extends StatelessWidget {
  final LutPreset lut;
  final bool isActive;
  final double intensity;
  final bool showBeforeAfter;
  final VoidCallback onTap;

  const _LutCard({
    required this.lut,
    required this.isActive,
    required this.intensity,
    required this.showBeforeAfter,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        decoration: BoxDecoration(
          color: AppTheme.surfaceVariant,
          borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
          border: Border.all(
            color: isActive ? AppTheme.primary : AppTheme.border,
            width: isActive ? 2.0 : 1.0,
          ),
          boxShadow: isActive
              ? [
                  BoxShadow(
                    color: AppTheme.primary.withOpacity(0.2),
                    blurRadius: 8,
                  ),
                ]
              : null,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Gradient preview strip
            Expanded(
              child: Stack(
                children: [
                  // LUT gradient preview
                  ClipRRect(
                    borderRadius: const BorderRadius.vertical(
                        top: Radius.circular(AppTheme.radiusMedium - 1)),
                    child: CustomPaint(
                      painter: _LutGradientPainter(
                        colors: lut.gradientStops,
                        showBeforeAfter: showBeforeAfter,
                      ),
                      size: Size.infinite,
                    ),
                  ),

                  // "Before" label if comparing
                  if (showBeforeAfter)
                    Positioned(
                      top: 4,
                      left: 4,
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 4, vertical: 1),
                        decoration: BoxDecoration(
                          color: Colors.black54,
                          borderRadius:
                              BorderRadius.circular(AppTheme.radiusSmall),
                        ),
                        child: const Text(
                          'BEFORE',
                          style: TextStyle(
                            fontSize: 8,
                            fontWeight: FontWeight.w700,
                            color: Colors.white70,
                            letterSpacing: 0.5,
                          ),
                        ),
                      ),
                    ),

                  // "Applied" badge
                  if (isActive)
                    Positioned(
                      top: 4,
                      right: 4,
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: AppTheme.primary,
                          borderRadius:
                              BorderRadius.circular(AppTheme.radiusSmall),
                        ),
                        child: Text(
                          'Applied ${intensity < 100 ? "${intensity.round()}%" : ""}',
                          style: const TextStyle(
                            fontSize: 8,
                            fontWeight: FontWeight.w700,
                            color: Colors.white,
                            letterSpacing: 0.3,
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),

            // Info area
            Padding(
              padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacing8, vertical: AppTheme.spacing8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    lut.name,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: isActive ? FontWeight.w700 : FontWeight.w600,
                      color: isActive ? AppTheme.primary : AppTheme.textPrimary,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 2),
                  Text(
                    lut.description,
                    style: TextStyle(
                      fontSize: 10,
                      color: AppTheme.textSecondary,
                      height: 1.3,
                    ),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 4),
                  // Category tag
                  Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                    decoration: BoxDecoration(
                      color: AppTheme.cardColor,
                      borderRadius:
                          BorderRadius.circular(AppTheme.radiusSmall),
                    ),
                    child: Text(
                      lut.category,
                      style: TextStyle(
                        fontSize: 9,
                        fontWeight: FontWeight.w500,
                        color: AppTheme.textDisabled,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ─── LUT Gradient Painter ─────────────────────────────────────────────

class _LutGradientPainter extends CustomPainter {
  final List<Color> colors;
  final bool showBeforeAfter;

  _LutGradientPainter({
    required this.colors,
    this.showBeforeAfter = false,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (showBeforeAfter) {
      // Left half: neutral gradient (before)
      final neutralPaint = Paint()
        ..shader = LinearGradient(
          begin: Alignment.centerLeft,
          end: Alignment.centerRight,
          colors: [
            const Color(0xFF1A1A1E),
            const Color(0xFF444455),
            const Color(0xFF888899),
            const Color(0xFFCCCCEE),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.width / 2, size.height));
      canvas.drawRect(
          Rect.fromLTWH(0, 0, size.width / 2, size.height), neutralPaint);

      // Right half: LUT gradient (after)
      final lutPaint = Paint()
        ..shader = LinearGradient(
          begin: Alignment.centerLeft,
          end: Alignment.centerRight,
          colors: colors,
        ).createShader(
            Rect.fromLTWH(size.width / 2, 0, size.width / 2, size.height));
      canvas.drawRect(
          Rect.fromLTWH(size.width / 2, 0, size.width / 2, size.height),
          lutPaint);

      // Center divider line
      final divPaint = Paint()
        ..color = Colors.white.withOpacity(0.6)
        ..strokeWidth = 1.5;
      canvas.drawLine(
        Offset(size.width / 2, 0),
        Offset(size.width / 2, size.height),
        divPaint,
      );
    } else {
      // Full LUT gradient
      final paint = Paint()
        ..shader = LinearGradient(
          begin: Alignment.centerLeft,
          end: Alignment.centerRight,
          colors: colors,
        ).createShader(Rect.fromLTWH(0, 0, size.width, size.height));
      canvas.drawRect(
          Rect.fromLTWH(0, 0, size.width, size.height), paint);
    }
  }

  @override
  bool shouldRepaint(covariant _LutGradientPainter oldDelegate) {
    return oldDelegate.colors != colors ||
        oldDelegate.showBeforeAfter != showBeforeAfter;
  }
}

// ─── Action Chip ──────────────────────────────────────────────────────

class _ActionChip extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  const _ActionChip({
    required this.icon,
    required this.label,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: AppTheme.primary.withOpacity(0.1),
      borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 14, color: AppTheme.primary),
              const SizedBox(width: 4),
              Text(
                label,
                style: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.primary,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ─── Toggle Chip ──────────────────────────────────────────────────────

class _ToggleChip extends StatelessWidget {
  final String label;
  final IconData icon;
  final bool active;
  final ValueChanged<bool> onChanged;

  const _ToggleChip({
    required this.label,
    required this.icon,
    required this.active,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => onChanged(!active),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
        decoration: BoxDecoration(
          color: active ? AppTheme.primary.withOpacity(0.15) : AppTheme.cardColor,
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          border: Border.all(
            color: active ? AppTheme.primary : AppTheme.border,
            width: 1,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon,
                size: 14, color: active ? AppTheme.primary : AppTheme.textSecondary),
            const SizedBox(width: 4),
            Text(
              label,
              style: TextStyle(
                fontSize: 11,
                fontWeight: active ? FontWeight.w600 : FontWeight.w400,
                color: active ? AppTheme.primary : AppTheme.textSecondary,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ─── Category Dropdown ────────────────────────────────────────────────

class _CategoryDropdown extends StatelessWidget {
  final List<String> categories;
  final String selected;
  final ValueChanged<String> onChanged;

  const _CategoryDropdown({
    required this.categories,
    required this.selected,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: AppTheme.cardColor,
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
        border: Border.all(color: AppTheme.border, width: 1),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<String>(
          value: selected,
          isDense: true,
          iconSize: 14,
          icon: const Icon(Icons.arrow_drop_down, color: AppTheme.textSecondary),
          style: const TextStyle(
            fontSize: 11,
            fontWeight: FontWeight.w500,
            color: AppTheme.textPrimary,
          ),
          items: categories
              .map((c) => DropdownMenuItem(
                    value: c,
                    child: Text(c),
                  ))
              .toList(),
          onChanged: (v) {
            if (v != null) onChanged(v);
          },
        ),
      ),
    );
  }
}
