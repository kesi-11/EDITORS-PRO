import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';

import '../../data/models/project_model.dart';

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
  }) {
    return ProjectState(
      currentProject: currentProject ?? this.currentProject,
      recentProjects: recentProjects ?? this.recentProjects,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

/// Project state notifier
class ProjectNotifier extends StateNotifier<ProjectState> {
  ProjectNotifier() : super(const ProjectState());

  final _uuid = const Uuid();

  /// Create a new project
  Future<void> createProject(String name, {int width = 1920, int height = 1080, double fps = 30.0}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final now = DateTime.now().millisecondsSinceEpoch;
      final project = ProjectModel(
        id: _uuid.v4(),
        name: name,
        createdAt: now,
        updatedAt: now,
        width: width,
        height: height,
        fps: fps,
        tracks: [
          TrackModel(
            id: _uuid.v4(),
            name: 'Video 1',
            trackType: TrackType.video,
            orderIndex: 0,
          ),
          TrackModel(
            id: _uuid.v4(),
            name: 'Audio 1',
            trackType: TrackType.audio,
            orderIndex: 1,
          ),
          TrackModel(
            id: _uuid.v4(),
            name: 'Text',
            trackType: TrackType.text,
            orderIndex: 2,
          ),
        ],
      );

      state = state.copyWith(
        currentProject: project,
        isLoading: false,
        recentProjects: [project, ...state.recentProjects],
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Open an existing project
  void openProject(ProjectModel project) {
    state = state.copyWith(currentProject: project);
  }

  /// Close the current project
  void closeProject() {
    state = state.copyWith(currentProject: null);
  }

  /// Import media into the current project
  Future<void> importMedia(String filePath, String fileName, MediaType mediaType, {
    int? durationMs, int? width, int? height, int fileSizeBytes = 0,
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
    );

    final updatedProject = state.currentProject!.copyWith(
      mediaAssets: [...state.currentProject!.mediaAssets, asset],
    );

    state = state.copyWith(currentProject: updatedProject);
  }

  /// Add a clip to a track
  void addClipToTrack(String trackId, ClipModel clip) {
    if (state.currentProject == null) return;

    final updatedTracks = state.currentProject!.tracks.map((track) {
      if (track.id == trackId) {
        return track.copyWith(clips: [...track.clips, clip]);
      }
      return track;
    }).toList();

    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(tracks: updatedTracks),
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
      currentProject: state.currentProject!.copyWith(tracks: updatedTracks),
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
      currentProject: state.currentProject!.copyWith(tracks: updatedTracks),
    );
  }

  /// Update the timeline duration
  void updateDuration(int durationMs) {
    if (state.currentProject == null) return;
    state = state.copyWith(
      currentProject: state.currentProject!.copyWith(durationMs: durationMs),
    );
  }
}

/// Provider for project state
final projectProvider = StateNotifierProvider<ProjectNotifier, ProjectState>((ref) {
  return ProjectNotifier();
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
