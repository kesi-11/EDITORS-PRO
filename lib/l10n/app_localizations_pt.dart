// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get appName => 'EDITORS-PRO';

  @override
  String get settingsTitle => 'Configurações';

  @override
  String get settingsAppearance => 'Aparência';

  @override
  String get settingsTheme => 'Tema';

  @override
  String get settingsThemeSystem => 'Seguir sistema';

  @override
  String get settingsThemeLight => 'Claro';

  @override
  String get settingsThemeDark => 'Escuro';

  @override
  String get settingsProjectDefaults => 'Padrões do projeto';

  @override
  String get settingsEditorBehavior => 'Comportamento do editor';

  @override
  String get settingsPerformance => 'Desempenho';

  @override
  String get settingsExport => 'Exportar';

  @override
  String get settingsStorage => 'Armazenamento';

  @override
  String get settingsCloudSync => 'Sincronização na nuvem';

  @override
  String get settingsPrivacyData => 'Privacidade e dados';

  @override
  String get settingsAbout => 'Sobre';

  @override
  String get settingsExperimental => 'Experimental';

  @override
  String get settingsExperimentalAutoCaptions => 'Legendas automáticas';

  @override
  String get settingsExperimentalAutoCaptionsDesc =>
      'Ativar UI de transcrição (simulada; Whisper chegará na Phase D)';

  @override
  String get settingsExperimentalCloudSync => 'Sincronização na nuvem';

  @override
  String get settingsExperimentalCloudSyncDesc =>
      'Mostrar aba Nuvem (placeholder; Google Drive chegará na Phase D)';

  @override
  String get settingsExperimentalAiBgRemoval => 'Remoção de fundo com IA';

  @override
  String get settingsExperimentalAiBgRemovalDesc =>
      'Ativar efeito U²-Net (não conectado; ONNX Runtime chegará na Phase D)';

  @override
  String get editorSplitAtPlayhead => 'Cortar no cursor';

  @override
  String get editorDeleteSelected => 'Excluir seleção';

  @override
  String get editorUndo => 'Desfazer';

  @override
  String get editorRedo => 'Refazer';

  @override
  String get editorSave => 'Salvar';

  @override
  String get editorProjectSaved => 'Projeto salvo';

  @override
  String get errorDismiss => 'Fechar';

  @override
  String get errorImportFailed => 'Falha na importação';

  @override
  String get errorSplitFailed => 'Falha no corte';

  @override
  String get errorDeleteFailed => 'Falha na exclusão';

  @override
  String get errorUndoFailed => 'Falha ao desfazer';

  @override
  String get errorRedoFailed => 'Falha ao refazer';

  @override
  String get commonCancel => 'Cancelar';

  @override
  String get commonConfirm => 'Confirmar';

  @override
  String get commonOK => 'OK';

  @override
  String get commonRetry => 'Tentar novamente';
}
