import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:go_router/go_router.dart';

import '../../../core/constants/app_icons.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/widgets/app_icon.dart';
import '../../../data/models/project_model.dart';
import '../providers/project_provider.dart';

/// Home screen showing project list, templates, and create button.
class ProjectHomeScreen extends ConsumerStatefulWidget {
  const ProjectHomeScreen({super.key});

  @override
  ConsumerState<ProjectHomeScreen> createState() => _ProjectHomeScreenState();
}

class _ProjectHomeScreenState extends ConsumerState<ProjectHomeScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    // Load persisted projects on startup
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(projectProvider.notifier).loadProjects();
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final projectState = ref.watch(projectProvider);

    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            _buildHeader(context),
            _buildTabBar(context),
            Expanded(
              child: TabBarView(
                controller: _tabController,
                children: [
                  _buildRecentProjects(context, projectState),
                  _buildTemplates(context),
                ],
              ),
            ),
          ],
        ),
      ),
      floatingActionButton: projectState.projects.isEmpty ? null : _buildCreateButton(context),
    );
  }

  // ─── Header ───────────────────────────────────────────────────
  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
      child: Row(
        children: [
          // Logo in 44x44 rounded container with subtle glow
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(12),
              boxShadow: AppTheme.primaryGlow(opacity: 0.35),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(12),
              child: Image.asset(
                'assets/icons/logo.png',
                fit: BoxFit.cover,
              ),
            ),
          ),
          const SizedBox(width: 12),
          // Title + subtitle
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'EDITORS-PRO',
                style: context.textTheme.titleLarge?.copyWith(
                  fontWeight: FontWeight.w800,
                  letterSpacing: 1.5,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                'Video Editor',
                style: context.textTheme.bodySmall?.copyWith(
                  color: AppTheme.textSecondary,
                  letterSpacing: 0.3,
                ),
              ),
            ],
          ),
          const Spacer(),
          // Search
          _buildHeaderIconButton(
            icon: AppIcons.search,
            onTap: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: const Text('Search coming soon!'),
                  behavior: SnackBarBehavior.floating,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(10),
                  ),
                  duration: const Duration(seconds: 2),
                ),
              );
            },
          ),
          const SizedBox(width: 8),
          // Settings
          _buildHeaderIconButton(
            icon: AppIcons.settings,
            onTap: () => context.go('/settings'),
          ),
        ],
      ),
    ).animate().fadeIn(duration: 400.ms).slideY(begin: -0.05, end: 0);
  }

  Widget _buildHeaderIconButton({
    required String icon,
    required VoidCallback onTap,
  }) {
    return Material(
      color: AppTheme.surfaceVariant,
      shape: const CircleBorder(),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Container(
          width: 40,
          height: 40,
          alignment: Alignment.center,
          child: AppIcon(icon, size: 18, color: AppTheme.textSecondary),
        ),
      ),
    );
  }

  // ─── Pill-style Tab Bar ───────────────────────────────────────
  Widget _buildTabBar(BuildContext context) {
    return Container(
      margin: const EdgeInsets.fromLTRB(20, 8, 20, 12),
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        color: AppTheme.surfaceVariant,
        borderRadius: BorderRadius.circular(AppTheme.radiusFull),
      ),
      child: TabBar(
        controller: _tabController,
        indicator: BoxDecoration(
          color: AppTheme.primary.withOpacity(0.18),
          borderRadius: BorderRadius.circular(AppTheme.radiusFull),
          border: Border.all(
            color: AppTheme.primary.withOpacity(0.4),
            width: 1,
          ),
        ),
        indicatorSize: TabBarIndicatorSize.tab,
        dividerColor: Colors.transparent,
        labelColor: AppTheme.primary,
        unselectedLabelColor: AppTheme.textSecondary,
        labelPadding: const EdgeInsets.symmetric(vertical: 10),
        labelStyle: const TextStyle(
          fontWeight: FontWeight.w700,
          fontSize: 13,
          letterSpacing: 1.4,
        ),
        unselectedLabelStyle: const TextStyle(
          fontWeight: FontWeight.w600,
          fontSize: 13,
          letterSpacing: 1.4,
        ),
        tabs: const [
          Tab(text: 'RECENT'),
          Tab(text: 'TEMPLATES'),
        ],
      ),
    );
  }

  // ─── Recent Projects Tab ──────────────────────────────────────
  Widget _buildRecentProjects(BuildContext context, ProjectState state) {
    if (state.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.recentProjects.isEmpty) {
      return _buildEmptyState(context);
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth >= 600;
        final crossAxisCount = isWide ? 2 : 1;
        return GridView.builder(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 96),
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: crossAxisCount,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: isWide ? 0.82 : 1.2,
          ),
          itemCount: state.recentProjects.length,
          itemBuilder: (context, index) {
            final project = state.recentProjects[index];
            return _ProjectCard(
              project: project,
              onTap: () async {
                await ref.read(projectProvider.notifier).openProject(project);
                if (context.mounted) {
                  context.go('/editor/${project.id}');
                }
              },
              onMore: () => _showProjectActions(context, project),
            )
                .animate(delay: (index * 80).ms)
                .fadeIn(duration: 400.ms)
                .slideY(begin: 0.15, end: 0, duration: 400.ms);
          },
        );
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Large film illustration in a soft circle
            Container(
              width: 144,
              height: 144,
              decoration: const BoxDecoration(
                color: AppTheme.surfaceVariant,
                shape: BoxShape.circle,
              ),
              alignment: Alignment.center,
              child: SvgPicture.asset(
                AppIcons.film,
                width: 80,
                height: 80,
                colorFilter: const ColorFilter.mode(
                  AppTheme.textDisabled,
                  BlendMode.srcIn,
                ),
              ),
            ).animate().scaleXY(
                  duration: 600.ms,
                  curve: Curves.elasticOut,
                  begin: 0.5,
                  end: 1.0,
                ),
            const SizedBox(height: 28),
            Text(
              'No Projects Yet',
              style: context.textTheme.headlineSmall?.copyWith(
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Create your first video project and\nstart editing like a pro',
              textAlign: TextAlign.center,
              style: context.textTheme.bodyMedium?.copyWith(
                color: AppTheme.textSecondary,
                height: 1.5,
              ),
            ),
            const SizedBox(height: 32),
            _GradientButton(
              icon: AppIcons.plus,
              label: 'Create Project',
              gradient: AppTheme.primaryGradient,
              shadow: AppTheme.primaryGlow(opacity: 0.45),
              onPressed: () => _showCreateProjectDialog(context),
            ),
          ],
        ),
      ),
    );
  }

  // ─── Templates Tab ────────────────────────────────────────────
  Widget _buildTemplates(BuildContext context) {
    final templates = <_TemplateData>[
      _TemplateData(
        name: 'Social Vertical',
        icon: AppIcons.phone,
        aspectRatio: '9:16',
        dimensions: '1080 × 1920',
        width: 1080,
        height: 1920,
        gradient: AppTheme.accentGradient,
        color: AppTheme.accent,
      ),
      _TemplateData(
        name: 'Social Square',
        icon: AppIcons.crop,
        aspectRatio: '1:1',
        dimensions: '1080 × 1080',
        width: 1080,
        height: 1080,
        gradient: AppTheme.secondaryGradient,
        color: AppTheme.secondary,
      ),
      _TemplateData(
        name: 'YouTube',
        icon: AppIcons.monitor,
        aspectRatio: '16:9',
        dimensions: '1920 × 1080',
        width: 1920,
        height: 1080,
        gradient: AppTheme.primaryGradient,
        color: AppTheme.primary,
      ),
      _TemplateData(
        name: 'Cinematic',
        icon: AppIcons.film,
        aspectRatio: '21:9',
        dimensions: '2560 × 1080',
        width: 2560,
        height: 1080,
        gradient: const LinearGradient(
          colors: [Color(0xFFFFB84D), Color(0xFFFF7E5F)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        color: AppTheme.warning,
      ),
    ];

    return GridView.builder(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 96),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 2,
        crossAxisSpacing: 16,
        mainAxisSpacing: 16,
        childAspectRatio: 1.0,
      ),
      itemCount: templates.length,
      itemBuilder: (context, index) {
        final t = templates[index];
        return _TemplateCard(
          data: t,
          onTap: () => _createProjectFromTemplate(
            context,
            t.name,
            width: t.width,
            height: t.height,
          ),
        )
            .animate(delay: (index * 80).ms)
            .fadeIn(duration: 400.ms)
            .scaleXY(
              begin: 0.9,
              end: 1.0,
              duration: 400.ms,
              curve: Curves.easeOutBack,
            );
      },
    );
  }

  // ─── Extended FAB ─────────────────────────────────────────────
  Widget _buildCreateButton(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        gradient: AppTheme.primaryGradient,
        borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
        boxShadow: AppTheme.primaryGlow(opacity: 0.5),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: () => _showCreateProjectDialog(context),
          borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
          child: const Padding(
            padding: EdgeInsets.symmetric(horizontal: 22, vertical: 16),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                AppIcon(AppIcons.plus, size: 22, color: Colors.white),
                SizedBox(width: 10),
                Text(
                  'New Project',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w700,
                    fontSize: 15,
                    letterSpacing: 0.3,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    ).animate().scaleXY(
          duration: 600.ms,
          delay: 500.ms,
          curve: Curves.elasticOut,
          begin: 0.5,
          end: 1.0,
        );
  }

  // ─── Dialogs & Project Ops ────────────────────────────────────
  void _showCreateProjectDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (dialogContext) => _CreateProjectDialog(
        onCreate: (name, width, height) {
          Navigator.pop(dialogContext);
          unawaited(_createProject(name, width: width, height: height));
        },
      ),
    );
  }

  void _createProjectFromTemplate(
    BuildContext context,
    String templateName, {
    int width = 1920,
    int height = 1080,
  }) {
    unawaited(_createProject(templateName, width: width, height: height));
  }

  Future<void> _createProject(
    String name, {
    int width = 1920,
    int height = 1080,
  }) async {
    final effectiveName = name.trim().isEmpty ? 'Untitled Project' : name.trim();
    await ref
        .read(projectProvider.notifier)
        .createProject(effectiveName, width: width, height: height);
    if (!mounted) return;
    final project = ref.read(projectProvider).currentProject;
    if (project != null) {
      context.go('/editor/${project.id}');
    }
  }

  void _showDeleteConfirmation(BuildContext context, ProjectModel project) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Delete "${project.name}"?'),
        content: const Text(
          'This will permanently delete the project and all its media. '
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
              unawaited(
                ref.read(projectProvider.notifier).deleteProject(project.id),
              );
            },
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  /// Phase F.2: Show a bottom sheet with project actions (Open, Duplicate,
  /// Delete). Replaces the old "more" button that only offered delete.
  /// This is the pro workflow pattern — duplicate to fork a delivery
  /// variant, delete to remove, open to edit.
  void _showProjectActions(BuildContext context, ProjectModel project) {
    showModalBottomSheet<void>(
      context: context,
      backgroundColor: AppTheme.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(
          top: Radius.circular(AppTheme.radiusXLarge),
        ),
      ),
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Handle
            Container(
              width: 40, height: 4,
              margin: const EdgeInsets.symmetric(vertical: 12),
              decoration: BoxDecoration(
                color: AppTheme.textDisabled,
                borderRadius: BorderRadius.circular(2),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacing16, vertical: AppTheme.spacing4),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  project.name,
                  style: Theme.of(context).textTheme.titleMedium,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
            const Divider(),
            ListTile(
              leading: const Icon(Icons.play_arrow),
              title: const Text('Open project'),
              subtitle: const Text('Continue editing in the timeline'),
              onTap: () {
                Navigator.pop(ctx);
                unawaited(
                  ref.read(projectProvider.notifier).openProject(project).then((_) {
                    if (context.mounted) {
                      context.go('/editor/${project.id}');
                    }
                  }),
                );
              },
            ),
            ListTile(
              leading: const Icon(Icons.content_copy),
              title: const Text('Duplicate project'),
              subtitle: const Text(
                'Create a copy with the same media for a delivery variant',
              ),
              onTap: () {
                Navigator.pop(ctx);
                unawaited(
                  ref.read(projectProvider.notifier).duplicateProject(project.id).then((_) {
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(
                          content: Text('"${project.name}" duplicated.'),
                          action: SnackBarAction(
                            label: 'Open copy',
                            onPressed: () {
                              // The duplicated project is at the top of recentProjects
                              final dup = ref.read(projectProvider).recentProjects.first;
                              context.go('/editor/${dup.id}');
                            },
                          ),
                        ),
                      );
                    }
                  }).catchError((e) {
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Duplicate failed: $e')),
                      );
                    }
                  }),
                );
              },
            ),
            ListTile(
              leading: Icon(Icons.delete_outline, color: AppTheme.error),
              title: Text('Delete project',
                  style: TextStyle(color: AppTheme.error)),
              subtitle: const Text(
                'Permanently remove the project and its media references',
              ),
              onTap: () {
                Navigator.pop(ctx);
                _showDeleteConfirmation(context, project);
              },
            ),
            const SizedBox(height: AppTheme.spacing8),
          ],
        ),
      ),
    );
  }
}

// ─── Project Card ────────────────────────────────────────────────
class _ProjectCard extends StatelessWidget {
  final ProjectModel project;
  final VoidCallback onTap;
  final VoidCallback onMore;

  const _ProjectCard({
    required this.project,
    required this.onTap,
    required this.onMore,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.cardColor,
        borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
        boxShadow: AppTheme.softShadow,
        border: Border.all(color: AppTheme.border.withOpacity(0.5)),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // ─ Thumbnail (16:9) with overlays ─
              Stack(
                children: [
                  AspectRatio(
                    aspectRatio: 16 / 9,
                    child: Container(
                      decoration: const BoxDecoration(
                        gradient: AppTheme.sunsetGradient,
                        borderRadius: BorderRadius.vertical(
                          top: Radius.circular(AppTheme.radiusLarge),
                        ),
                      ),
                      child: Center(
                        child: Container(
                          width: 44,
                          height: 44,
                          decoration: BoxDecoration(
                            color: Colors.black.withOpacity(0.32),
                            shape: BoxShape.circle,
                          ),
                          alignment: Alignment.center,
                          child: AppIcon(
                            AppIcons.play,
                            size: 20,
                            color: Colors.white,
                          ),
                        ),
                      ),
                    ),
                  ),
                  // More options button (top-right)
                  Positioned(
                    top: 8,
                    right: 8,
                    child: GestureDetector(
                      onTap: onMore,
                      behavior: HitTestBehavior.opaque,
                      child: Container(
                        width: 32,
                        height: 32,
                        decoration: BoxDecoration(
                          color: Colors.black.withOpacity(0.45),
                          shape: BoxShape.circle,
                        ),
                        alignment: Alignment.center,
                        child: AppIcon(
                          AppIcons.moreHorizontal,
                          size: 16,
                          color: Colors.white,
                        ),
                      ),
                    ),
                  ),
                  // Duration badge (bottom-right)
                  Positioned(
                    bottom: 8,
                    right: 8,
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: Colors.black.withOpacity(0.65),
                        borderRadius:
                            BorderRadius.circular(AppTheme.radiusSmall),
                      ),
                      child: Text(
                        _formatDuration(project.durationMs),
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 11,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 0.3,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
              // ─ Info section ─
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 12, 12, 14),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      project.name,
                      style: context.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${project.width}×${project.height} • ${project.fps.toInt()}fps',
                      style: context.textTheme.bodySmall?.copyWith(
                        color: AppTheme.textSecondary,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 6),
                    Text(
                      'Edited ${_formatDate(project.updatedAt)}',
                      style: context.textTheme.labelSmall?.copyWith(
                        color: AppTheme.textDisabled,
                        letterSpacing: 0.3,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatDuration(int durationMs) {
    if (durationMs <= 0) return '0:00';
    final duration = Duration(milliseconds: durationMs);
    final minutes = duration.inMinutes;
    final seconds = duration.inSeconds.remainder(60);
    return '$minutes:${seconds.toString().padLeft(2, '0')}';
  }

  String _formatDate(int timestamp) {
    final date = DateTime.fromMillisecondsSinceEpoch(timestamp);
    return '${date.day}/${date.month}/${date.year}';
  }
}

// ─── Template Data + Card ────────────────────────────────────────
class _TemplateData {
  final String name;
  final String icon;
  final String aspectRatio;
  final String dimensions;
  final int width;
  final int height;
  final Gradient gradient;
  final Color color;

  const _TemplateData({
    required this.name,
    required this.icon,
    required this.aspectRatio,
    required this.dimensions,
    required this.width,
    required this.height,
    required this.gradient,
    required this.color,
  });
}

class _TemplateCard extends StatelessWidget {
  final _TemplateData data;
  final VoidCallback onTap;

  const _TemplateCard({required this.data, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.cardColor,
        borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
        boxShadow: AppTheme.softShadow,
        border: Border.all(color: AppTheme.border.withOpacity(0.5)),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Icon in colored gradient circle
                Container(
                  width: 52,
                  height: 52,
                  decoration: BoxDecoration(
                    gradient: data.gradient,
                    shape: BoxShape.circle,
                    boxShadow: [
                      BoxShadow(
                        color: data.color.withOpacity(0.4),
                        blurRadius: 12,
                        offset: const Offset(0, 4),
                      ),
                    ],
                  ),
                  alignment: Alignment.center,
                  child: AppIcon(data.icon, size: 24, color: Colors.white),
                ),
                const SizedBox(height: 14),
                Text(
                  data.name,
                  style: context.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  data.aspectRatio,
                  style: context.textTheme.bodySmall?.copyWith(
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  data.dimensions,
                  style: context.textTheme.labelSmall?.copyWith(
                    color: AppTheme.textDisabled,
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

// ─── Reusable Gradient Button ────────────────────────────────────
class _GradientButton extends StatelessWidget {
  final String icon;
  final String label;
  final Gradient gradient;
  final List<BoxShadow>? shadow;
  final VoidCallback onPressed;
  final EdgeInsets padding;

  const _GradientButton({
    required this.icon,
    required this.label,
    required this.gradient,
    required this.onPressed,
    this.shadow,
    this.padding = const EdgeInsets.symmetric(horizontal: 22, vertical: 14),
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        gradient: gradient,
        borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
        boxShadow: shadow,
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onPressed,
          borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
          child: Padding(
            padding: padding,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                AppIcon(icon, size: 18, color: Colors.white),
                const SizedBox(width: 8),
                Text(
                  label,
                  style: const TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w700,
                    fontSize: 14,
                    letterSpacing: 0.3,
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

// ─── Create Project Dialog ───────────────────────────────────────
class _CreateProjectDialog extends StatefulWidget {
  final void Function(String name, int width, int height) onCreate;

  const _CreateProjectDialog({required this.onCreate});

  @override
  State<_CreateProjectDialog> createState() => _CreateProjectDialogState();
}

class _CreateProjectDialogState extends State<_CreateProjectDialog> {
  final _nameController = TextEditingController(text: 'Untitled Project');
  String _selectedResolution = '1080p';

  static const _resolutions = <_ResolutionOption>[
    _ResolutionOption('720p', 1280, 720),
    _ResolutionOption('1080p', 1920, 1080),
    _ResolutionOption('4K', 3840, 2160),
  ];

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: AppTheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
      ),
      title: Row(
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              gradient: AppTheme.primaryGradient,
              borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
              boxShadow: AppTheme.primaryGlow(opacity: 0.4),
            ),
            alignment: Alignment.center,
            child: AppIcon(AppIcons.plus, size: 20, color: Colors.white),
          ),
          const SizedBox(width: 12),
          const Text('New Project'),
        ],
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Project Name',
            style: context.textTheme.labelLarge?.copyWith(
              color: AppTheme.textSecondary,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.4,
            ),
          ),
          const SizedBox(height: 8),
          TextField(
            controller: _nameController,
            decoration: const InputDecoration(
              hintText: 'Enter project name',
            ),
            autofocus: true,
            textCapitalization: TextCapitalization.words,
          ),
          const SizedBox(height: 20),
          Text(
            'Resolution',
            style: context.textTheme.labelLarge?.copyWith(
              color: AppTheme.textSecondary,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.4,
            ),
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: _resolutions.map((option) {
              final isSelected = _selectedResolution == option.label;
              return GestureDetector(
                onTap: () => setState(() => _selectedResolution = option.label),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 200),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    gradient: isSelected ? AppTheme.primaryGradient : null,
                    color: isSelected ? null : AppTheme.surfaceVariant,
                    borderRadius:
                        BorderRadius.circular(AppTheme.radiusMedium),
                    border: Border.all(
                      color: isSelected
                          ? AppTheme.primaryLight
                          : AppTheme.border,
                      width: 1.5,
                    ),
                    boxShadow:
                        isSelected ? AppTheme.primaryGlow(opacity: 0.35) : null,
                  ),
                  child: Text(
                    option.label,
                    style: TextStyle(
                      color:
                          isSelected ? Colors.white : AppTheme.textSecondary,
                      fontWeight: FontWeight.w700,
                      fontSize: 13,
                      letterSpacing: 0.3,
                    ),
                  ),
                ),
              );
            }).toList(),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        _GradientButton(
          icon: AppIcons.plus,
          label: 'Create',
          gradient: AppTheme.primaryGradient,
          shadow: AppTheme.primaryGlow(opacity: 0.4),
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          onPressed: () {
            final selected = _resolutions.firstWhere(
              (r) => r.label == _selectedResolution,
            );
            widget.onCreate(
              _nameController.text,
              selected.width,
              selected.height,
            );
          },
        ),
      ],
    );
  }
}

class _ResolutionOption {
  final String label;
  final int width;
  final int height;
  const _ResolutionOption(this.label, this.width, this.height);
}
