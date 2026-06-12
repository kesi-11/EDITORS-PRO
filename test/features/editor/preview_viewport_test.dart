import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:editors_pro/core/services/engine_service.dart';
import 'package:editors_pro/core/services/project_repository.dart';
import 'package:editors_pro/data/models/project_model.dart';
import 'package:editors_pro/features/editor/providers/editor_provider.dart';
import 'package:editors_pro/features/editor/widgets/preview_viewport.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('PreviewViewport', () {
    setUp(() {
      EngineService.instance.dispose();
    });

    tearDown(() {
      EngineService.instance.dispose();
    });

    /// Build the PreviewViewport wrapped in a MaterialApp + ProviderScope
    /// with all required providers overridden for testing.
    Future<void> pumpViewport(
      WidgetTester tester, {
      EditorState editorState = const EditorState(),
      ProjectModel? project,
    }) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            // Provide a fake project repository so database isn't needed.
            projectRepositoryProvider
                .overrideWith((ref) => _FakeProjectRepository()),

            // Override the editor provider so we control the state.
            editorProvider.overrideWith((ref) {
              return _TestEditorNotifier(editorState);
            }),
          ],
          child: MaterialApp(
            theme: ThemeData.dark(),
            home: const Scaffold(
              body: SizedBox(
                width: 800,
                height: 600,
                child: PreviewViewport(),
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));
    }

    testWidgets('renders without crashing', (tester) async {
      await pumpViewport(tester);
      expect(find.byType(PreviewViewport), findsOneWidget);
    });

    testWidgets('shows placeholder when no frame data', (tester) async {
      await pumpViewport(tester);

      // The placeholder shows a movie icon and "Preview" text.
      expect(find.byIcon(Icons.movie_creation_outlined), findsOneWidget);
      expect(find.text('Preview'), findsOneWidget);
    });

    testWidgets('shows play button when not playing', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(isPlaying: false),
      );

      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    });

    testWidgets('does not show play button when playing', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(isPlaying: true),
      );

      expect(find.byIcon(Icons.play_arrow), findsNothing);
    });

    testWidgets('shows playback speed indicator when speed != 1.0',
        (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(playbackSpeed: 2.0),
      );

      expect(find.text('2.0x'), findsOneWidget);
    });

    testWidgets('does not show speed indicator when speed is 1.0',
        (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(playbackSpeed: 1.0),
      );

      expect(find.text('1.0x'), findsNothing);
    });

    testWidgets('shows current time in scrub bar', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(currentTimeMs: 5000, durationMs: 30000),
      );

      // 5000ms → 00:05.00
      expect(find.textContaining('00:05'), findsWidgets);
    });

    testWidgets('shows duration in scrub bar', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(currentTimeMs: 0, durationMs: 30000),
      );

      // 30000ms → 00:30.00
      expect(find.textContaining('00:30'), findsWidgets);
    });

    testWidgets('tapping play button triggers playback toggle', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(isPlaying: false),
      );

      final playButton = find.byIcon(Icons.play_arrow);
      expect(playButton, findsOneWidget);

      await tester.tap(playButton);
      // The tap should be handled without throwing.
    });

    testWidgets('placeholder displays formatted time', (tester) async {
      await pumpViewport(
        tester,
        editorState: const EditorState(currentTimeMs: 65000),
      );

      // 65000ms → 01:05.00
      expect(find.textContaining('01:05'), findsWidgets);
    });
  });
}

/// A [EditorNotifier] that starts with a given [EditorState] and does
/// not try to initialize the Rust engine.
class _TestEditorNotifier extends StateNotifier<EditorState> {
  _TestEditorNotifier(EditorState initialState) : super(initialState);

  @override
  Future<void> initialize() async {
    // No-op in tests
  }

  void togglePlayback() {
    state = state.copyWith(isPlaying: !state.isPlaying);
  }

  void seekTo(int timeMs) {
    state = state.copyWith(currentTimeMs: timeMs);
  }

  void selectClip(String? clipId) {
    state = state.copyWith(
      selectedClipId: clipId,
      showInspector: clipId != null,
    );
  }

  void zoomIn() {
    state = state.copyWith(zoomLevel: (state.zoomLevel * 1.2).clamp(0.1, 10.0));
  }

  void zoomOut() {
    state = state.copyWith(zoomLevel: (state.zoomLevel / 1.2).clamp(0.1, 10.0));
  }
}

/// A lightweight fake repository that satisfies [ProjectRepository].
class _FakeProjectRepository implements ProjectRepository {
  @override
  Future<ProjectModel> createProject(
    String name, {
    int width = 1920,
    int height = 1080,
    double fps = 30.0,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    return ProjectModel(
      id: 'fake',
      name: name,
      createdAt: now,
      updatedAt: now,
      width: width,
      height: height,
      fps: fps,
    );
  }

  @override
  Future<List<ProjectModel>> getAllProjects() async => [];

  @override
  Future<ProjectModel?> getProject(String id) async => null;

  @override
  Future<void> updateProject(ProjectModel project) async {}

  @override
  Future<void> saveProjectToEngine(ProjectModel project) async {}

  @override
  Future<void> loadProjectFromEngine(String eppFilePath) async {}

  @override
  Future<void> addMediaAsset(String projectId, MediaAssetModel asset) async {}

  @override
  Future<void> deleteProject(String projectId) async {}
}
