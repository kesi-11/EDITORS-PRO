// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for French (`fr`).
class AppLocalizationsFr extends AppLocalizations {
  AppLocalizationsFr([String locale = 'fr']) : super(locale);

  @override
  String get appName => 'EDITORS-PRO';

  @override
  String get settingsTitle => 'Paramètres';

  @override
  String get settingsAppearance => 'Apparence';

  @override
  String get settingsTheme => 'Thème';

  @override
  String get settingsThemeSystem => 'Suivre le système';

  @override
  String get settingsThemeLight => 'Clair';

  @override
  String get settingsThemeDark => 'Sombre';

  @override
  String get settingsProjectDefaults => 'Paramètres du projet';

  @override
  String get settingsEditorBehavior => 'Comportement de l\'éditeur';

  @override
  String get settingsPerformance => 'Performance';

  @override
  String get settingsExport => 'Exportation';

  @override
  String get settingsStorage => 'Stockage';

  @override
  String get settingsCloudSync => 'Synchronisation cloud';

  @override
  String get settingsPrivacyData => 'Confidentialité et données';

  @override
  String get settingsAbout => 'À propos';

  @override
  String get settingsExperimental => 'Expérimental';

  @override
  String get settingsExperimentalAutoCaptions => 'Sous-titres automatiques';

  @override
  String get settingsExperimentalAutoCaptionsDesc =>
      'Activer l\'UI de transcription (simulée ; Whisper arrivera en Phase D)';

  @override
  String get settingsExperimentalCloudSync => 'Synchronisation cloud';

  @override
  String get settingsExperimentalCloudSyncDesc =>
      'Afficher l\'onglet Cloud (placeholder ; Google Drive arrivera en Phase D)';

  @override
  String get settingsExperimentalAiBgRemoval =>
      'Suppression d\'arrière-plan IA';

  @override
  String get settingsExperimentalAiBgRemovalDesc =>
      'Activer l\'effet U²-Net (non connecté ; ONNX Runtime arrivera en Phase D)';

  @override
  String get editorSplitAtPlayhead => 'Couper au curseur';

  @override
  String get editorDeleteSelected => 'Supprimer la sélection';

  @override
  String get editorUndo => 'Annuler';

  @override
  String get editorRedo => 'Rétablir';

  @override
  String get editorSave => 'Enregistrer';

  @override
  String get editorProjectSaved => 'Projet enregistré';

  @override
  String get errorDismiss => 'Fermer';

  @override
  String get errorImportFailed => 'Échec de l\'importation';

  @override
  String get errorSplitFailed => 'Échec de la coupe';

  @override
  String get errorDeleteFailed => 'Échec de la suppression';

  @override
  String get errorUndoFailed => 'Échec de l\'annulation';

  @override
  String get errorRedoFailed => 'Échec du rétablissement';

  @override
  String get commonCancel => 'Annuler';

  @override
  String get commonConfirm => 'Confirmer';

  @override
  String get commonOK => 'OK';

  @override
  String get commonRetry => 'Réessayer';
}
