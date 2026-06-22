// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get appName => 'EDITORS-PRO';

  @override
  String get settingsTitle => 'Ajustes';

  @override
  String get settingsAppearance => 'Apariencia';

  @override
  String get settingsTheme => 'Tema';

  @override
  String get settingsThemeSystem => 'Seguir sistema';

  @override
  String get settingsThemeLight => 'Claro';

  @override
  String get settingsThemeDark => 'Oscuro';

  @override
  String get settingsProjectDefaults => 'Predeterminados del proyecto';

  @override
  String get settingsEditorBehavior => 'Comportamiento del editor';

  @override
  String get settingsPerformance => 'Rendimiento';

  @override
  String get settingsExport => 'Exportar';

  @override
  String get settingsStorage => 'Almacenamiento';

  @override
  String get settingsCloudSync => 'Sincronización en la nube';

  @override
  String get settingsPrivacyData => 'Privacidad y datos';

  @override
  String get settingsAbout => 'Acerca de';

  @override
  String get settingsExperimental => 'Experimental';

  @override
  String get settingsExperimentalAutoCaptions => 'Subtítulos automáticos';

  @override
  String get settingsExperimentalAutoCaptionsDesc =>
      'Habilitar IU de transcripción (simulada; Whisper llegará en Phase D)';

  @override
  String get settingsExperimentalCloudSync => 'Sincronización en la nube';

  @override
  String get settingsExperimentalCloudSyncDesc =>
      'Mostrar pestaña Nube (placeholder; Google Drive llegará en Phase D)';

  @override
  String get settingsExperimentalAiBgRemoval => 'Eliminación de fondo con IA';

  @override
  String get settingsExperimentalAiBgRemovalDesc =>
      'Habilitar efecto U²-Net (no conectado; ONNX Runtime llegará en Phase D)';

  @override
  String get editorSplitAtPlayhead => 'Dividir en cabezal';

  @override
  String get editorDeleteSelected => 'Eliminar selección';

  @override
  String get editorUndo => 'Deshacer';

  @override
  String get editorRedo => 'Rehacer';

  @override
  String get editorSave => 'Guardar';

  @override
  String get editorProjectSaved => 'Proyecto guardado';

  @override
  String get errorDismiss => 'Cerrar';

  @override
  String get errorImportFailed => 'Importación fallida';

  @override
  String get errorSplitFailed => 'División fallida';

  @override
  String get errorDeleteFailed => 'Eliminación fallida';

  @override
  String get errorUndoFailed => 'Deshacer fallido';

  @override
  String get errorRedoFailed => 'Rehacer fallido';

  @override
  String get commonCancel => 'Cancelar';

  @override
  String get commonConfirm => 'Confirmar';

  @override
  String get commonOK => 'OK';

  @override
  String get commonRetry => 'Reintentar';
}
