import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:editors_pro/app.dart';
import 'package:editors_pro/core/services/engine_service.dart';
import 'package:editors_pro/core/services/project_repository.dart';
import 'package:editors_pro/data/models/project_model.dart';

/// A lightweight fake that satisfies the [ProjectRepository] interface
/// without touching the filesystem or SQLite.
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
      id: 'fake-${DateTime.now().millisecondsSinceEpoch}',
      name: name,
      createdAt: now,
      updatedAt: now,
      width: width,
      height: height,
      fps: fps,
      tracks: [
        TrackModel(id: 'v1', name: 'Video 1', trackType: TrackType.video, orderIndex: 0),
        TrackModel(id: 'a1', name: 'Audio 1', trackType: TrackType.audio, orderIndex: 1),
        TrackModel(id: 't1', name: 'Text', trackType: TrackType.text, orderIndex: 2),
      ],
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

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('EditorsProApp', () {
    setUp(() {
      EngineService.instance.dispose();
    });

    tearDown(() {
      EngineService.instance.dispose();
    });

    /// Pump the app with a fake repository so database calls don't crash.
    Future<void> pumpApp(WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            projectRepositoryProvider
                .overrideWith((ref) => _FakeProjectRepository()),
          ],
          child: const EditorsProApp(),
        ),
      );
      // Give GoRouter + async providers a frame to settle.
      await tester.pump(const Duration(milliseconds: 300));
    }

    testWidgets('renders MaterialApp without crashing', (tester) async {
      await pumpApp(tester);
      expect(find.byType(MaterialApp), findsOneWidget);
    });

    testWidgets('shows the project home screen', (tester) async {
      await pumpApp(tester);
      // The home screen shows the app title in the header.
      expect(find.text('EDITORS-PRO'), findsOneWidget);
    });

    testWidgets('dark theme is applied', (tester) async {
      await pumpApp(tester);

      final materialApp = tester.widget<MaterialApp>(
        find.byType(MaterialApp),
      );

      final theme = materialApp.theme;
      expect(theme, isNotNull);
      expect(theme!.brightness, equals(Brightness.dark));
    });

    testWidgets('settings button navigates to settings route', (tester) async {
      await pumpApp(tester);

      final settingsButton = find.byIcon(Icons.settings_outlined);
      expect(settingsButton, findsOneWidget);

      await tester.tap(settingsButton);
      await tester.pumpAndSettle(const Duration(seconds: 2));

      // The SettingsScreen AppBar title is "Settings".
      expect(find.text('Settings'), findsWidgets);
    });

    testWidgets('navigating to editor route does not crash', (tester) async {
      await pumpApp(tester);

      // Push the editor route programmatically via GoRouter.
      final element = tester.element(find.byType(MaterialApp).first);
      final container = ProviderScope.containerOf(element);
      final router = container.read(routerProvider);

      router.go('/editor/test-project-id');
      await tester.pumpAndSettle(const Duration(seconds: 2));

      // Even without the engine the editor screen should render a Scaffold.
      expect(find.byType(Scaffold), findsWidgets);
    });

    testWidgets('app title is EDITORS-PRO', (tester) async {
      await pumpApp(tester);

      final materialApp = tester.widget<MaterialApp>(
        find.byType(MaterialApp),
      );
      expect(materialApp.title, equals('EDITORS-PRO'));
    });

    testWidgets('debug banner is hidden', (tester) async {
      await pumpApp(tester);

      final materialApp = tester.widget<MaterialApp>(
        find.byType(MaterialApp),
      );
      expect(materialApp.debugShowCheckedModeBanner, isFalse);
    });

    testWidgets('home screen shows RECENT and TEMPLATES tabs', (tester) async {
      await pumpApp(tester);

      expect(find.text('RECENT'), findsOneWidget);
      expect(find.text('TEMPLATES'), findsOneWidget);
    });

    testWidgets('home screen shows empty state when no projects', (tester) async {
      await pumpApp(tester);

      expect(find.text('No Projects Yet'), findsOneWidget);
    });

    testWidgets('home screen has New Project FAB', (tester) async {
      await pumpApp(tester);

      expect(find.text('New Project'), findsOneWidget);
    });
  });
}
