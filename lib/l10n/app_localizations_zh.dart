// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appName => 'EDITORS-PRO';

  @override
  String get settingsTitle => '设置';

  @override
  String get settingsAppearance => '外观';

  @override
  String get settingsTheme => '主题';

  @override
  String get settingsThemeSystem => '跟随系统';

  @override
  String get settingsThemeLight => '浅色';

  @override
  String get settingsThemeDark => '深色';

  @override
  String get settingsProjectDefaults => '项目默认设置';

  @override
  String get settingsEditorBehavior => '编辑器行为';

  @override
  String get settingsPerformance => '性能';

  @override
  String get settingsExport => '导出';

  @override
  String get settingsStorage => '存储';

  @override
  String get settingsCloudSync => '云同步';

  @override
  String get settingsPrivacyData => '隐私和数据';

  @override
  String get settingsAbout => '关于';

  @override
  String get settingsExperimental => '实验性功能';

  @override
  String get settingsExperimentalAutoCaptions => '自动字幕';

  @override
  String get settingsExperimentalAutoCaptionsDesc =>
      '启用转录界面（模拟；真正的 Whisper 将在 Phase D 推出）';

  @override
  String get settingsExperimentalCloudSync => '云同步';

  @override
  String get settingsExperimentalCloudSyncDesc =>
      '显示云标签页（占位符；Google Drive 将在 Phase D 推出）';

  @override
  String get settingsExperimentalAiBgRemoval => 'AI 背景移除';

  @override
  String get settingsExperimentalAiBgRemovalDesc =>
      '启用 U²-Net 效果（尚未连接；ONNX Runtime 将在 Phase D 推出）';

  @override
  String get editorSplitAtPlayhead => '在播放头处分割';

  @override
  String get editorDeleteSelected => '删除所选';

  @override
  String get editorUndo => '撤销';

  @override
  String get editorRedo => '重做';

  @override
  String get editorSave => '保存';

  @override
  String get editorProjectSaved => '项目已保存';

  @override
  String get errorDismiss => '关闭';

  @override
  String get errorImportFailed => '导入失败';

  @override
  String get errorSplitFailed => '分割失败';

  @override
  String get errorDeleteFailed => '删除失败';

  @override
  String get errorUndoFailed => '撤销失败';

  @override
  String get errorRedoFailed => '重做失败';

  @override
  String get commonCancel => '取消';

  @override
  String get commonConfirm => '确认';

  @override
  String get commonOK => '确定';

  @override
  String get commonRetry => '重试';
}
