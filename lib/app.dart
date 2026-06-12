import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'core/theme/app_theme.dart';
import 'features/projects/presentation/project_home_screen.dart';
import 'features/editor/presentation/editor_screen.dart';
import 'features/export/presentation/export_screen.dart';
import 'features/settings/settings_screen.dart';
import 'features/onboarding/presentation/onboarding_screen.dart';
import 'features/splash/splash_screen.dart';
import 'features/cloud/presentation/cloud_screen.dart';
import 'features/templates/presentation/template_browser.dart';

final routerProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/splash',
    debugLogDiagnostics: true,
    routes: [
      GoRoute(
        path: '/splash',
        name: 'splash',
        builder: (context, state) => const SplashScreen(),
      ),
      GoRoute(
        path: '/onboarding',
        name: 'onboarding',
        builder: (context, state) => const OnboardingScreen(),
      ),
      GoRoute(
        path: '/',
        name: 'home',
        builder: (context, state) => const ProjectHomeScreen(),
      ),
      GoRoute(
        path: '/editor/:projectId',
        name: 'editor',
        builder: (context, state) {
          final projectId = state.pathParameters['projectId'] ?? '';
          return EditorScreen(projectId: projectId);
        },
      ),
      GoRoute(
        path: '/export/:projectId',
        name: 'export',
        builder: (context, state) {
          final projectId = state.pathParameters['projectId'] ?? '';
          return ExportScreen(projectId: projectId);
        },
      ),
      GoRoute(
        path: '/settings',
        name: 'settings',
        builder: (context, state) => const SettingsScreen(),
      ),
      GoRoute(
        path: '/cloud',
        name: 'cloud',
        builder: (context, state) => const CloudScreen(),
      ),
      GoRoute(
        path: '/templates',
        name: 'templates',
        builder: (context, state) => const TemplateBrowserScreen(),
      ),
    ],
  );
});

class EditorsProApp extends ConsumerWidget {
  const EditorsProApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);

    return MaterialApp.router(
      title: 'EDITORS-PRO',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.darkTheme,
      routerConfig: router,
    );
  }
}
