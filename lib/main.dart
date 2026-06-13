import 'dart:developer' as developer;
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'app.dart';
import 'core/services/engine_service.dart';
import 'core/services/performance_service.dart';
import 'features/onboarding/providers/onboarding_provider.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // ─── Flutter error handler ────────────────────────────────
  FlutterError.onError = (details) {
    FlutterError.presentError(details);
    developer.log(
      'Flutter error: ${details.exceptionAsString()}',
      name: 'FlutterError',
      error: details.exception,
      stackTrace: details.stack,
    );
  };

  // ─── Async error handler ──────────────────────────────────
  PlatformDispatcher.instance.onError = (error, stack) {
    developer.log(
      'Uncaught async error: $error',
      name: 'AsyncError',
      error: error,
      stackTrace: stack,
    );
    return true;
  };

  _initializeAndRun();
}

Future<void> _initializeAndRun() async {
  final perf = PerformanceService.instance..markAppStart();

  // Initialise SharedPreferences eagerly so it is available for
  // the ProviderScope override.
  final sharedPreferences = await SharedPreferences.getInstance();

  try {
    final service = EngineService.instance;
    final success = await service.initialize();

    if (!success) {
      developer.log(
        'Engine initialization failed — app will run in degraded mode',
        name: 'main',
      );
    }

    perf.markEngineReady();

    developer.log(
      'Cold start → engine ready in ${perf.coldStartDuration?.inMilliseconds} ms',
      name: 'Performance',
    );
  } catch (e, st) {
    developer.log(
      'Engine initialization error: $e',
      name: 'main',
      error: e,
      stackTrace: st,
    );
  }

  // Always run the app. Widgets that depend on the engine will
  // gracefully handle the case where it is unavailable.
  runApp(
    ProviderScope(
      overrides: [
        sharedPreferencesProvider.overrideWithValue(sharedPreferences),
      ],
      child: const EditorsProApp(),
    ),
  );
}
