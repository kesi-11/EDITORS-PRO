import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:editors_pro/core/services/engine_service.dart';
import 'package:editors_pro/src/rust/api/bridge_api.dart';

/// Mock implementation of the EditorsProEngineApi for testing.
///
/// Returns deterministic data without requiring the native Rust engine.
/// This allows testing the Flutter UI and state management layer
/// in isolation from the engine.
class MockEditorsProEngineApi extends EditorsProEngineApi {
  bool _initialized = false;
  ProjectInfo? _currentProject;
  final List<TrackInfo> _tracks = [];
  final List<ClipInfo> _clips = [];
  final List<MediaAssetInfo> _assets = [];
  final List<String> _undoStack = [];

  @override
  Future<void> initialize() async {
    _initialized = true;
  }

  @override
  Future<ProjectInfo> createProject({
    required String name,
    BridgeProjectSettings? settings,
  }) async {
    final project = ProjectInfo(
      id: 'mock-proj-${DateTime.now().millisecondsSinceEpoch}',
      name: name,
      createdAt: DateTime.now().millisecondsSinceEpoch,
      updatedAt: DateTime.now().millisecondsSinceEpoch,
      width: settings?.width ?? 1920,
      height: settings?.height ?? 1080,
      fps: settings?.fps ?? 30.0,
      durationMs: BigInt.zero,
      trackCount: 0,
      clipCount: 0,
      assetCount: 0,
    );
    _currentProject = project;
    _tracks.clear();
    _clips.clear();
    _assets.clear();
    return project;
  }

  @override
  Future<MediaAssetInfo> importMedia({required String filePath}) async {
    final asset = MediaAssetInfo(
      id: 'mock-asset-${_assets.length}',
      filePath: filePath,
      fileName: filePath.split('/').last,
      mediaType: 'Video',
      durationMs: BigInt.from(10000),
      width: 1920,
      height: 1080,
      fileSizeBytes: BigInt.from(5242880),
    );
    _assets.add(asset);
    return asset;
  }

  @override
  Future<TrackInfo> addTrack({
    required String trackType,
    String? name,
  }) async {
    final track = TrackInfo(
      id: 'mock-track-${_tracks.length}',
      name: name ?? '$trackType ${_tracks.length + 1}',
      trackType: trackType,
      clipCount: 0,
      locked: false,
      visible: true,
      volume: 1.0,
    );
    _tracks.add(track);
    return track;
  }

  @override
  Future<ClipInfo> addClip({
    required String trackId,
    required String assetId,
    required BigInt startMs,
    required BigInt durationMs,
  }) async {
    final clip = ClipInfo(
      id: 'mock-clip-${_clips.length}',
      assetId: assetId,
      startMs: startMs,
      durationMs: durationMs != BigInt.zero ? durationMs : BigInt.from(5000),
      trimStartMs: BigInt.zero,
      trimEndMs: BigInt.zero,
      speed: 1.0,
      opacity: 1.0,
      effectsCount: 0,
      hasTransitionIn: false,
      hasTransitionOut: false,
    );
    _clips.add(clip);
    return clip;
  }

  @override
  Future<BigInt> getTimelineDuration() async {
    if (_clips.isEmpty) return BigInt.zero;
    final maxEnd = _clips.map((c) => c.startMs + c.durationMs).reduce(
      (a, b) => a > b ? a : b,
    );
    return maxEnd;
  }

  @override
  Future<ProjectInfo?> getProjectInfo() async => _currentProject;

  @override
  Future<Uint8List> getFrame({required BigInt timeMs}) async {
    // Return a 4x4 black PNG as a placeholder
    // This is a minimal valid PNG file
    return Uint8List.fromList([
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
      0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
      0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, // 4x4
      0x08, 0x02, 0x00, 0x00, 0x00, // 8-bit RGB
      0xFF, 0x90, 0x45, 0x0E, // CRC
      0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT
      0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0x00, 0x00,
      0x00, 0x04, 0x00, 0x01, // compressed black pixels
      0xF6, 0x17, 0xA4, 0x49, // CRC
      0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
      0xAE, 0x42, 0x60, 0x82, // CRC
    ]);
  }

  @override
  Future<void> undo() async {
    if (_undoStack.isNotEmpty) {
      _undoStack.removeLast();
    }
  }

  @override
  Future<void> redo() async {
    // No-op in mock
  }

  @override
  Future<bool> canUndo() async => _undoStack.isNotEmpty;

  @override
  Future<bool> canRedo() async => false;

  @override
  Future<List<FilterTypeInfo>> getFilterCatalog() async => [
        const FilterTypeInfo(
          name: 'Brightness',
          icon: 'wb_sunny',
          parameters: [
            EffectParameterInfo(
              name: 'brightness',
              displayName: 'Brightness',
              value: 0.0,
              minValue: -1.0,
              maxValue: 1.0,
              defaultValue: 0.0,
              step: 0.01,
            ),
          ],
        ),
        const FilterTypeInfo(
          name: 'Contrast',
          icon: 'contrast',
          parameters: [
            EffectParameterInfo(
              name: 'contrast',
              displayName: 'Contrast',
              value: 0.0,
              minValue: -1.0,
              maxValue: 1.0,
              defaultValue: 0.0,
              step: 0.01,
            ),
          ],
        ),
      ];

  @override
  Future<List<TransitionTypeInfo>> getTransitionCatalog() async => [
        const TransitionTypeInfo(
          name: 'Fade',
          icon: 'gradient',
          defaultDurationMs: BigInt.from(500),
        ),
      ];

  @override
  Future<List<double>> getWaveform({
    required String assetId,
    required int numBins,
  }) async {
    return List.generate(numBins, (i) => (i % 10) / 10.0);
  }
}

void main() {
  group('MockEditorsProEngineApi', () {
    late MockEditorsProEngineApi api;

    setUp(() {
      api = MockEditorsProEngineApi();
    });

    test('initialize succeeds', () async {
      await api.initialize();
      // No exception means success
    });

    test('createProject returns valid ProjectInfo', () async {
      final project = await api.createProject(name: 'Test');

      expect(project.id, isNotEmpty);
      expect(project.name, equals('Test'));
      expect(project.width, equals(1920));
      expect(project.height, equals(1080));
      expect(project.fps, equals(30.0));
    });

    test('createProject with custom settings', () async {
      final project = await api.createProject(
        name: '4K Project',
        settings: const BridgeProjectSettings(
          width: 3840,
          height: 2160,
          fps: 60.0,
        ),
      );

      expect(project.width, equals(3840));
      expect(project.height, equals(2160));
      expect(project.fps, equals(60.0));
    });

    test('importMedia returns valid MediaAssetInfo', () async {
      final asset = await api.importMedia(filePath: '/path/to/video.mp4');

      expect(asset.id, isNotEmpty);
      expect(asset.fileName, equals('video.mp4'));
      expect(asset.mediaType, equals('Video'));
      expect(asset.durationMs, equals(BigInt.from(10000)));
    });

    test('addTrack returns valid TrackInfo', () async {
      final track = await api.addTrack(trackType: 'Video', name: 'Main');

      expect(track.id, isNotEmpty);
      expect(track.name, equals('Main'));
      expect(track.trackType, equals('Video'));
      expect(track.visible, isTrue);
    });

    test('addClip returns valid ClipInfo', () async {
      await api.createProject(name: 'Test');
      await api.addTrack(trackType: 'Video');

      final clip = await api.addClip(
        trackId: 'track-0',
        assetId: 'asset-0',
        startMs: BigInt.zero,
        durationMs: BigInt.from(5000),
      );

      expect(clip.id, isNotEmpty);
      expect(clip.startMs, equals(BigInt.zero));
      expect(clip.durationMs, equals(BigInt.from(5000)));
    });

    test('getTimelineDuration returns max clip end', () async {
      await api.createProject(name: 'Test');
      await api.addClip(
        trackId: 't1',
        assetId: 'a1',
        startMs: BigInt.from(1000),
        durationMs: BigInt.from(5000),
      );
      await api.addClip(
        trackId: 't1',
        assetId: 'a2',
        startMs: BigInt.from(8000),
        durationMs: BigInt.from(3000),
      );

      final duration = await api.getTimelineDuration();
      expect(duration, equals(BigInt.from(11000)));
    });

    test('getTimelineDuration returns zero for empty timeline', () async {
      final duration = await api.getTimelineDuration();
      expect(duration, equals(BigInt.zero));
    });

    test('getFrame returns PNG bytes', () async {
      final frame = await api.getFrame(timeMs: BigInt.from(1000));
      expect(frame, isNotNull);
      expect(frame.length, greaterThan(8));
      // PNG signature starts with 89 50 4E 47
      expect(frame[0], equals(0x89));
      expect(frame[1], equals(0x50));
    });

    test('getFilterCatalog returns built-in filters', () async {
      final catalog = await api.getFilterCatalog();
      expect(catalog, isNotEmpty);
      expect(catalog.any((f) => f.name == 'Brightness'), isTrue);
      expect(catalog.any((f) => f.name == 'Contrast'), isTrue);
    });

    test('getTransitionCatalog returns built-in transitions', () async {
      final catalog = await api.getTransitionCatalog();
      expect(catalog, isNotEmpty);
      expect(catalog.any((t) => t.name == 'Fade'), isTrue);
    });

    test('getWaveform returns list of doubles', () async {
      final waveform = await api.getWaveform(assetId: 'test', numBins: 50);
      expect(waveform, hasLength(50));
      expect(waveform.every((v) => v >= 0.0 && v <= 1.0), isTrue);
    });

    test('undo/redo work correctly', () async {
      expect(await api.canUndo(), isFalse);
      expect(await api.canRedo(), isFalse);

      // Simulate an undo-able action
      api._undoStack.add('action1');
      expect(await api.canUndo(), isTrue);

      await api.undo();
      expect(await api.canUndo(), isFalse);
    });
  });
}
