import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../../../core/theme/app_theme.dart';
import '../../../core/extensions/context_extensions.dart';
import '../../../data/models/project_model.dart';
import '../providers/project_provider.dart';

/// Home screen showing project list and create button
class ProjectHomeScreen extends ConsumerStatefulWidget {
  const ProjectHomeScreen({super.key});

  @override
  ConsumerState<ProjectHomeScreen> createState() => _ProjectHomeScreenState();
}

class _ProjectHomeScreenState extends ConsumerState<ProjectHomeScreen> with SingleTickerProviderStateMixin {
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
            // Header
            _buildHeader(context),

            // Tab bar
            Container(
              decoration: const BoxDecoration(
                border: Border(bottom: BorderSide(color: Color(0xFF2A2A3E))),
              ),
              child: TabBar(
                controller: _tabController,
                tabs: const [
                  Tab(text: 'RECENT'),
                  Tab(text: 'TEMPLATES'),
                ],
              ),
            ),

            // Content
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
      floatingActionButton: _buildCreateButton(context),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 20, 20, 12),
      child: Row(
        children: [
          // Logo & Title
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(10),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withOpacity(0.3),
                  blurRadius: 8,
                  offset: const Offset(0, 2),
                ),
              ],
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(10),
              child: Image.asset(
                'assets/icons/logo.png',
                fit: BoxFit.cover,
              ),
            ),
          ),
          const SizedBox(width: 12),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'EDITORS-PRO',
                style: context.textTheme.headlineSmall?.copyWith(
                  fontWeight: FontWeight.w800,
                  letterSpacing: 1.5,
                ),
              ),
              Text(
                'Video Editor',
                style: context.textTheme.bodySmall?.copyWith(
                  color: AppTheme.textSecondary,
                ),
              ),
            ],
          ),
          const Spacer(),
          // Settings button
          IconButton(
            onPressed: () => context.go('/settings'),
            icon: const Icon(Icons.settings_outlined),
            style: IconButton.styleFrom(
              backgroundColor: AppTheme.surfaceVariant,
            ),
          ),
        ],
      ),
    ).animate().fadeIn(duration: 400.ms);
  }

  Widget _buildRecentProjects(BuildContext context, ProjectState state) {
    if (state.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.recentProjects.isEmpty) {
      return _buildEmptyState(context);
    }

    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
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
          onDelete: () => _showDeleteConfirmation(context, project),
        ).animate().slideX(
          begin: 0.1,
          duration: 300.ms,
          delay: (index * 50).ms,
        );
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(40),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.video_library_outlined,
              size: 80,
              color: AppTheme.textDisabled,
            ).animate().scale(duration: 500.ms),
            const SizedBox(height: 24),
            Text(
              'No Projects Yet',
              style: context.textTheme.headlineSmall?.copyWith(
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Create your first video project\nand start editing like a pro.',
              textAlign: TextAlign.center,
              style: context.textTheme.bodyMedium?.copyWith(
                color: AppTheme.textDisabled,
              ),
            ),
            const SizedBox(height: 32),
            ElevatedButton.icon(
              onPressed: () => _showCreateProjectDialog(context),
              icon: const Icon(Icons.add),
              label: const Text('Create Project'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTemplates(BuildContext context) {
    final templates = [
      ('Social Vertical', '9:16 for TikTok & Reels', Icons.phone_android, AppTheme.accent, 1080, 1920),
      ('Social Square', '1:1 for Instagram', Icons.crop_square, AppTheme.secondary, 1080, 1080),
      ('YouTube', '16:9 standard', Icons.play_circle, AppTheme.primary, 1920, 1080),
      ('Cinematic', '21:9 widescreen', Icons.theaters, AppTheme.warning, 2560, 1080),
    ];

    return GridView.builder(
      padding: const EdgeInsets.all(16),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 2,
        childAspectRatio: 1.2,
        crossAxisSpacing: 12,
        mainAxisSpacing: 12,
      ),
      itemCount: templates.length,
      itemBuilder: (context, index) {
        final (name, description, icon, color, width, height) = templates[index];
        return _TemplateCard(
          name: name,
          description: description,
          icon: icon,
          color: color,
          onTap: () => _createProjectFromTemplate(context, name, width: width, height: height),
        ).animate().scale(
          duration: 300.ms,
          delay: (index * 80).ms,
        );
      },
    );
  }

  Widget _buildCreateButton(BuildContext context) {
    return FloatingActionButton.extended(
      onPressed: () => _showCreateProjectDialog(context),
      backgroundColor: AppTheme.primary,
      icon: const Icon(Icons.add, color: Colors.white),
      label: const Text(
        'New Project',
        style: TextStyle(color: Colors.white, fontWeight: FontWeight.w600),
      ),
    ).animate().scale(duration: 300.ms, delay: 500.ms);
  }

  void _showCreateProjectDialog(BuildContext context) {
    final nameController = TextEditingController(text: 'Untitled Project');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New Project'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(
                labelText: 'Project Name',
                hintText: 'Enter project name',
              ),
              autofocus: true,
            ),
            const SizedBox(height: 16),
            // Resolution selector
            _ResolutionSelector(),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              _createProject(nameController.text);
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  void _createProjectFromTemplate(BuildContext context, String templateName, {int width = 1920, int height = 1080}) {
    _createProject(templateName, width: width, height: height);
  }

  void _createProject(String name, {int width = 1920, int height = 1080}) {
    ref.read(projectProvider.notifier).createProject(name, width: width, height: height);
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
              ref.read(projectProvider.notifier).deleteProject(project.id);
            },
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}

class _ProjectCard extends StatelessWidget {
  final ProjectModel project;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const _ProjectCard({
    required this.project,
    required this.onTap,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              // Thumbnail
              Container(
                width: 72,
                height: 48,
                decoration: BoxDecoration(
                  color: AppTheme.surfaceVariant,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                  gradient: const LinearGradient(
                    colors: [AppTheme.primary, AppTheme.secondary],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                ),
                child: const Icon(Icons.play_circle, color: Colors.white54, size: 24),
              ),
              const SizedBox(width: 16),
              // Info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      project.name,
                      style: context.textTheme.titleMedium,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${project.width}x${project.height} @ ${project.fps}fps',
                      style: context.textTheme.bodySmall,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      _formatDate(project.updatedAt),
                      style: context.textTheme.labelSmall,
                    ),
                  ],
                ),
              ),
              // Actions
              IconButton(
                onPressed: onDelete,
                icon: const Icon(Icons.delete_outline, size: 20),
                style: IconButton.styleFrom(
                  foregroundColor: AppTheme.error,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatDate(int timestamp) {
    final date = DateTime.fromMillisecondsSinceEpoch(timestamp);
    return '${date.day}/${date.month}/${date.year}';
  }
}

class _TemplateCard extends StatelessWidget {
  final String name;
  final String description;
  final IconData icon;
  final Color color;
  final VoidCallback onTap;

  const _TemplateCard({
    required this.name,
    required this.description,
    required this.icon,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: color.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(icon, color: color, size: 24),
              ),
              const SizedBox(height: 12),
              Text(
                name,
                style: context.textTheme.titleSmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 4),
              Text(
                description,
                style: context.textTheme.bodySmall,
                textAlign: TextAlign.center,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ResolutionSelector extends StatefulWidget {
  @override
  State<_ResolutionSelector> createState() => _ResolutionSelectorState();
}

class _ResolutionSelectorState extends State<_ResolutionSelector> {
  String _selected = '1080p';

  final _resolutions = {
    '720p': (1280, 720),
    '1080p': (1920, 1080),
    '4K': (3840, 2160),
  };

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Resolution', style: context.textTheme.titleSmall),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          children: _resolutions.entries.map((entry) {
            final isSelected = _selected == entry.key;
            return ChoiceChip(
              label: Text(entry.key),
              selected: isSelected,
              onSelected: (_) => setState(() => _selected = entry.key),
            );
          }).toList(),
        ),
      ],
    );
  }
}
