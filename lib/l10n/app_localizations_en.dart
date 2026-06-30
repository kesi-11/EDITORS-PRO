// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'EDITORS-PRO';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsThemeSystem => 'Follow system';

  @override
  String get settingsThemeLight => 'Light';

  @override
  String get settingsThemeDark => 'Dark';

  @override
  String get settingsProjectDefaults => 'Project Defaults';

  @override
  String get settingsEditorBehavior => 'Editor Behavior';

  @override
  String get settingsPerformance => 'Performance';

  @override
  String get settingsExport => 'Export';

  @override
  String get settingsStorage => 'Storage';

  @override
  String get settingsCloudSync => 'Cloud Sync';

  @override
  String get settingsPrivacyData => 'Privacy & Data';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsExperimental => 'Experimental';

  @override
  String get settingsExperimentalAutoCaptions => 'Auto Captions';

  @override
  String get settingsExperimentalAutoCaptionsDesc =>
      'Enable transcription UI (currently simulated; real Whisper coming in Phase D)';

  @override
  String get settingsExperimentalCloudSync => 'Cloud Sync';

  @override
  String get settingsExperimentalCloudSyncDesc =>
      'Show the Cloud tab (placeholder backend; Google Drive coming in Phase D)';

  @override
  String get settingsExperimentalAiBgRemoval => 'AI Background Removal';

  @override
  String get settingsExperimentalAiBgRemovalDesc =>
      'Enable U²-Net effect (not yet wired; ONNX Runtime coming in Phase D)';

  @override
  String get editorSplitAtPlayhead => 'Split at playhead';

  @override
  String get editorDeleteSelected => 'Delete selected';

  @override
  String get editorUndo => 'Undo';

  @override
  String get editorRedo => 'Redo';

  @override
  String get editorSave => 'Save';

  @override
  String get editorProjectSaved => 'Project saved';

  @override
  String get errorDismiss => 'Dismiss';

  @override
  String get errorImportFailed => 'Import failed';

  @override
  String get errorSplitFailed => 'Split failed';

  @override
  String get errorDeleteFailed => 'Delete failed';

  @override
  String get errorUndoFailed => 'Undo failed';

  @override
  String get errorRedoFailed => 'Redo failed';

  @override
  String get commonCancel => 'Cancel';

  @override
  String get commonConfirm => 'Confirm';

  @override
  String get commonOK => 'OK';

  @override
  String get commonRetry => 'Retry';
}
