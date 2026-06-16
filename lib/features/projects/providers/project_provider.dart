import 'dart:developer' as developer;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';

import '../../../core/services/engine_service.dart';
import '../../../core/services/project_repository.dart';
import '../../../data/models/project_model.dart';

/// Current project state
class ProjectState {
  final ProjectModel? currentProject;
  final List<ProjectModel> recentProjects;
  final bool isLoading;
  final String? error;

  const ProjectState({
    this.currentProject,
    this.recentProjects = const [],
    this.isLoading = false,
    this.error,
  });

  ProjectState copyWith({
    ProjectModel? currentProject,
    List<ProjectModel>? recentProjects,
    bool? isLoading,
    String? error,
    bool clearError = false,
  }) {
    return ProjectState(
      currentProject: currentProject ?? this.currentProject,
      recentProjects: recentProjects ?? this.recentProjects,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

/// Project state notifier — now backed by the Drift database via
/// [ProjectRepository] for persistent storage.
class ProjectNotifier extends StateNotifier<ProjectState> {
  final Ref _ref;
  final _uuid = const Uuid();

  ProjectNotifier(this._ref) : super(const ProjectState());

  ProjectRepository get _repo => _ref.read(projectRepositoryProvider);

  /// Load all projects from the database on app startup.
  Future<void> loadProjects() async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final projects = await _repo.getAllProjects();
      state = state.copyWith(
        recentProjects: projects,
        isLoading: false,
      );
    } catch (e) {
      developer.log('loadProjects failed: $e', name: 'ProjectNotifier');
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Create a new project — persists to database and initializes in the
  /// Rust engine.
  Future<void> createProject(String name, {int width = 1920, int height = 1080, double fps = 30.0}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final project = await _repo.createProject(name, width: width, height: height, fps: fps);

      state = state.copyWith(
        currentProject: project,
        recentProjects: [project, ...state.recentProjects],
        isLoading: false,
      );
    } catch (e) {
      developer.log('createProject failed: $e', name: 'ProjectNotifier');
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Open an existing project — loads from DB and restores in engine.
  Future<void> openProject(ProjectModel project) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      // Load full project with assets from DB
      final fullProject = await _repo.getProject(project.id);
      if (fullProject != null) {
        // Try to restore from .epp file if available
        // (The DB row has eppFilePath if the project was previously saved)
        state = state.copyWith(
          currentProject: fullProject,
          isLoading: false,
        );
      } else {
        state = state.copyWith(
          currentProject: project,
          isLoading: false,
        );
      }
    } catch (e) {
      developer.log('openProject failed: $e', name: 'ProjectNotifier');
      state = state.copyWith(
        currentProject: project,
        isLoading: false,
      );
    }
  }

  /// Close the current project
  void closeProject() {
    state = state.copyWith(currentProject: null);
  }

  /// Delete a project — removes from DB and engine.
  Future<void> deleteProject(String projectId) async {
    try {
      await _repo.deleteProject(projectId);
      final updatedRecent = state.recentProjects.where((p) => p.id != projectId).toList();
      state = state.copyWith(
        recentProjects: updatedRecent,
        currentProject: state.currentProject?.id == projectId ? null : state.currentProject,
      );
    } catch (e) {
      developer.log('deleteProject failed: $e', name: 'ProjectNotifier');
      state = state.copyWith(error: 'Delete failed: $e');
    }
  }

  /// Import media into the current project — persists to DB + engine.
  Future<void> importMedia(String filePath, String fileName, MediaType mediaType, {
    int? durationMs, int? width, int? height, int fileSizeBytes = 0,
    String? codec, int? bitrate,
  }) async {
    if (state.currentProject == null) return;

    final asset = MediaAssetModel(
      id: _uuid.v4(),
      filePath: filePath,
      fileName: fileName,
      mediaType: mediaType,
      durationMs: durationMs,
      width: width,
      height: height,
      fileSizeBytes: fileSizeBytes,
      codec: codec,
      bitrate: bitrate,
    );

    // Persist to database
    await _repo.addMediaAsset(state.currentProject!.id, asset);

    final updatedProject = state.currentProject!.copyWith(
      mediaAssets: [...state.currentProject!.mediaAssets, asset],
      updatedAt: DateTime.now().millisecondsSinceEpoch,
    );

    state = state.copyWith(currentProject: updatedProject);
  }

  /// Add a clip to a track — updates the Flutter model (engine handles
  /// the actual clip creation via EditorNotifier).
  void addClipToTrack(String trackId, ClipModel clip) {
    if (state.currentProject == null) return;

    final updatedTracks = state.currentProject!.tracks.map((track) {
      if (track.id == trackId) {
        return track.copyWith(clips: [...track.clips, clip]);
      }
      return track;
    }).toList();

    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(
        tracks: updatedTracks,
        updatedAt: DateTime.now().millisecondsSinceEpoch,
      ),
    );
  }

  /// Update a track's properties (e.g., locked, visible, volume).
  void updateTrack(String trackId, TrackModel updatedTrack) {
    if (state.currentProject == null) return;

    final updatedTracks = state.currentProject!.tracks.map((track) {
      return track.id == trackId ? updatedTrack : track;
    }).toList();

    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(
        tracks: updatedTracks,
        updatedAt: DateTime.now().millisecondsSinceEpoch,
      ),
    );
  }

  /// Update a clip
  void updateClip(String clipId, ClipModel updatedClip) {
    if (state.currentProject == null) return;

    final updatedTracks = state.currentProject!.tracks.map((track) {
      final updatedClips = track.clips.map((clip) {
        return clip.id == clipId ? updatedClip : clip;
      }).toList();
      return track.copyWith(clips: updatedClips);
    }).toList();

    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(
        tracks: updatedTracks,
        updatedAt: DateTime.now().millisecondsSinceEpoch,
      ),
    );
  }

  /// Remove a clip
  void removeClip(String clipId) {
    if (state.currentProject == null) return;

    final updatedTracks = state.currentProject!.tracks.map((track) {
      final updatedClips = track.clips.where((c) => c.id != clipId).toList();
      return track.copyWith(clips: updatedClips);
    }).toList();

    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(
        tracks: updatedTracks,
        updatedAt: DateTime.now().millisecondsSinceEpoch,
      ),
    );
  }

  /// Update the timeline duration
  void updateDuration(int durationMs) {
    if (state.currentProject == null) return;
    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(durationMs: durationMs),
    );
  }

  /// Save the current project to the engine (.epp format).
  Future<void> saveCurrentProject() async {
    if (state.currentProject == null) return;
    try {
      await _repo.updateProject(state.currentProject!);
      await _repo.saveProjectToEngine(state.currentProject!);
    } catch (e) {
      developer.log('saveCurrentProject failed: $e', name: 'ProjectNotifier');
    }
  }

  /// Sync the project model with the latest engine state (after
  /// engine mutations like addClip, split, etc.).
  Future<void> syncFromEngine() async {
    if (!EngineService.instance.isInitialized || state.currentProject == null) return;
    try {
      final api = EngineService.instance.api;
      final timelineState = await api.getTimelineState();
      if (timelineState != null) {
        // Update duration from engine
        final duration = await api.getTimelineDuration();
        state = state.copyWith(
          currentProject: state.currentProject!.copyWith(
            durationMs: duration.toInt(),
            updatedAt: DateTime.now().millisecondsSinceEpoch,
          ),
        );
      }
    } catch (e) {
      developer.log('syncFromEngine failed: $e', name: 'ProjectNotifier');
    }
  }
}

/// Provider for project state
final projectProvider = StateNotifierProvider<ProjectNotifier, ProjectState>((ref) {
  return ProjectNotifier(ref);
});

/// Provider for current project
final currentProjectProvider = Provider<ProjectModel?>((ref) {
  return ref.watch(projectProvider).currentProject;
});

/// Provider for tracks in the current project
final tracksProvider = Provider<List<TrackModel>>((ref) {
  final project = ref.watch(currentProjectProvider);
  return project?.tracks ?? [];
});

/// Provider for media assets in the current project
final mediaAssetsProvider = Provider<List<MediaAssetModel>>((ref) {
  final project = ref.watch(currentProjectProvider);
  return project?.mediaAssets ?? [];
});
