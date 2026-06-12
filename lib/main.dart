import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'core/services/engine_service.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize the Rust engine before running the app.
  // We use a zone so that any initialization errors are caught and
  // the app still starts (in a degraded state) rather than crashing.
  _initializeAndRun();
}

Future<void> _initializeAndRun() async {
  try {
    final service = EngineService.instance;
    final success = await service.initialize();

    if (!success) {
      developer.log(
        'Engine initialization failed — app will run in degraded mode',
        name: 'main',
      );
    }
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
  runApp(const ProviderScope(child: EditorsProApp()));
}
