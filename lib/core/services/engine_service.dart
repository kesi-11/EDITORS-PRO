import 'dart:async';
import 'dart:developer' as developer;

import 'package:editors_pro/src/rust/api/bridge_api.dart';
import 'package:editors_pro/src/rust/frb_generated.dart';

/// Singleton service that manages the Rust engine lifecycle.
///
/// Lazily initializes [RustLib] and creates an [EditorsProEngineApi]
/// instance the first time it is accessed. All subsequent calls reuse
/// the same API instance.
///
/// Usage:
/// ```dart
/// final service = EngineService.instance;
/// await service.initialize();
/// final api = service.api;
/// ```
class EngineService {
  EngineService._();

  static final EngineService instance = EngineService._();

  EditorsProEngineApi? _api;
  bool _initialized = false;
  bool _initializing = false;
  Completer<void>? _initCompleter;

  /// Whether the engine has been successfully initialized.
  bool get isInitialized => _initialized;

  /// The bridge API instance.
  ///
  /// Throws [StateError] if the engine has not been initialized yet.
  EditorsProEngineApi get api {
    if (_api == null) {
      throw StateError(
        'EngineService not initialized. Call initialize() first.',
      );
    }
    return _api!;
  }

  /// Initialize the Rust engine.
  ///
  /// This must be called once before any engine operations. It is safe
  /// to call multiple times — subsequent calls will await the same
  /// initialization or return immediately if already done.
  ///
  /// Returns `true` if initialization succeeded, `false` otherwise.
  Future<bool> initialize() async {
    if (_initialized) return true;

    // If initialization is already in progress, wait for it.
    if (_initializing && _initCompleter != null) {
      await _initCompleter!.future;
      return _initialized;
    }

    _initializing = true;
    _initCompleter = Completer<void>();

    try {
      developer.log('Initializing RustLib…', name: 'EngineService');

      // Initialize the flutter_rust_bridge runtime.
      await RustLib.init();

      developer.log(
        'RustLib initialized, creating engine API…',
        name: 'EngineService',
      );

      // Get the API wrapper from the RustLib instance (which was
      // initialized above).  The noOp fallback is used when the
      // native library is not available.
      final engineApi = RustLib.instance.api;

      // If the engine is in noOp mode, skip native initialization.
      if (engineApi.isEngineAvailable) {
        await engineApi.initialize();
      } else {
        developer.log(
          'Engine is in noOp mode — skipping native initialization',
          name: 'EngineService',
        );
        // Don't mark as initialized since the engine isn't really available.
        _initializing = false;
        return false;
      }

      _api = engineApi;
      _initialized = true;

      developer.log(
        'Engine fully initialized',
        name: 'EngineService',
      );

      if (!(_initCompleter?.isCompleted ?? true)) {
        _initCompleter!.complete();
      }

      return true;
    } catch (e, st) {
      developer.log(
        'Engine initialization failed: $e',
        name: 'EngineService',
        error: e,
        stackTrace: st,
      );

      if (!(_initCompleter?.isCompleted ?? true)) {
        _initCompleter!.completeError(e, st);
      }

      _initializing = false;
      return false;
    }
  }

  /// Dispose of the engine and release resources.
  ///
  /// After calling this, the engine must be re-initialized before use.
  void dispose() {
    _api = null;
    _initialized = false;
    _initializing = false;
    _initCompleter = null;
  }
}
