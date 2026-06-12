/// Application-wide constants
class AppConstants {
  AppConstants._();

  static const String appName = 'EDITORS-PRO';
  static const String appVersion = '0.1.0';
  static const String projectFileExtension = '.epp';

  // Timeline defaults
  static const double defaultTimelineZoom = 1.0;
  static const double minTimelineZoom = 0.1;
  static const double maxTimelineZoom = 10.0;
  static const double timelinePixelsPerMs = 0.1;
  static const int defaultProjectFps = 30;
  static const int defaultProjectWidth = 1920;
  static const int defaultProjectHeight = 1080;

  // Export presets
  static const int export720pBitrate = 5000;
  static const int export1080pBitrate = 10000;
  static const int export4kBitrate = 40000;

  // UI
  static const double previewAspectRatio = 16 / 9;
  static const double timelineMinHeight = 160;
  static const double trackHeight = 48;
  static const double clipMinWidth = 20;
  static const double playheadWidth = 2;

  // Storage
  static const String projectsDir = 'projects';
  static const String mediaDir = 'media';
  static const String thumbnailsDir = 'thumbnails';
  static const String exportsDir = 'exports';
  static const String proxiesDir = 'proxies';
}
