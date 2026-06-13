/// Cloud sync state and Riverpod provider.
///
/// Manages the cloud sync lifecycle: authentication, project sync,
/// conflict resolution, and status tracking.  This is a FOUNDATION
/// implementation — actual OAuth2 and cloud API calls are placeholders.

import 'dart:developer' as developer;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/services/engine_service.dart';

// ─── Cloud Sync State ─────────────────────────────────────────────

/// Immutable snapshot of the cloud sync state.
class CloudSyncState {
  /// Whether the user is authenticated with a cloud provider.
  final bool isAuthenticated;

  /// The authenticated account name / email, if signed in.
  final String? accountName;

  /// The selected cloud provider's display name.
  final String providerName;

  /// Whether a sync operation is currently in progress.
  final bool isSyncing;

  /// Number of conflicts requiring resolution.
  final int pendingConflicts;

  /// Error message from the last failed operation, if any.
  final String? lastError;

  /// List of cloud project entries.
  final List<CloudProjectEntry> cloudProjects;

  const CloudSyncState({
    this.isAuthenticated = false,
    this.accountName,
    this.providerName = 'None',
    this.isSyncing = false,
    this.pendingConflicts = 0,
    this.lastError,
    this.cloudProjects = const [],
  });

  CloudSyncState copyWith({
    bool? isAuthenticated,
    String? accountName,
    String? providerName,
    bool? isSyncing,
    int? pendingConflicts,
    String? lastError,
    bool clearError = false,
    List<CloudProjectEntry>? cloudProjects,
  }) {
    return CloudSyncState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      accountName: accountName ?? this.accountName,
      providerName: providerName ?? this.providerName,
      isSyncing: isSyncing ?? this.isSyncing,
      pendingConflicts: pendingConflicts ?? this.pendingConflicts,
      lastError: clearError ? null : (lastError ?? this.lastError),
      cloudProjects: cloudProjects ?? this.cloudProjects,
    );
  }
}

/// A single cloud project entry for display in the UI.
class CloudProjectEntry {
  final String projectId;
  final String name;
  final int modifiedAt;
  final int sizeBytes;
  final String cloudFileId;
  final String providerName;

  const CloudProjectEntry({
    required this.projectId,
    required this.name,
    required this.modifiedAt,
    required this.sizeBytes,
    required this.cloudFileId,
    required this.providerName,
  });

  /// Format the file size for display (e.g., "1.2 MB").
  String get formattedSize {
    if (sizeBytes < 1024) return '$sizeBytes B';
    if (sizeBytes < 1024 * 1024) {
      return '${(sizeBytes / 1024).toStringAsFixed(1)} KB';
    }
    return '${(sizeBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  /// Format the modification date for display.
  String get formattedDate {
    final dt = DateTime.fromMillisecondsSinceEpoch(modifiedAt);
    return '${dt.year}-${dt.month.toString().padLeft(2, '0')}-${dt.day.toString().padLeft(2, '0')} '
        '${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}';
  }
}

// ─── Conflict Info ────────────────────────────────────────────────

/// Describes a sync conflict that the user needs to resolve.
class SyncConflictInfo {
  final String projectId;
  final String projectName;
  final int localModifiedAt;
  final int cloudModifiedAt;
  final String localChecksum;
  final String cloudChecksum;
  final String suggestedStrategy;

  const SyncConflictInfo({
    required this.projectId,
    required this.projectName,
    required this.localModifiedAt,
    required this.cloudModifiedAt,
    required this.localChecksum,
    required this.cloudChecksum,
    required this.suggestedStrategy,
  });
}

// ─── Cloud Sync Notifier ──────────────────────────────────────────

/// Riverpod notifier that manages the cloud sync lifecycle.
///
/// All cloud I/O is currently placeholder — the actual OAuth2 flows
/// and API calls will be implemented in a future phase.
class CloudSyncNotifier extends StateNotifier<CloudSyncState> {
  CloudSyncNotifier() : super(const CloudSyncState());

  /// Authenticate with the selected cloud provider.
  ///
  /// This is a placeholder — actual OAuth2 integration will be
  /// added when real providers are implemented.
  Future<void> authenticate() async {
    state = state.copyWith(isSyncing: true, clearError: true);

    try {
      developer.log(
        'Cloud authentication requested (placeholder)',
        name: 'CloudSyncNotifier',
      );

      // Placeholder — simulate authentication failure
      await Future.delayed(const Duration(milliseconds: 500));

      state = state.copyWith(
        isSyncing: false,
        lastError: 'Cloud sync not yet implemented',
      );
    } catch (e) {
      developer.log(
        'Cloud authentication failed: $e',
        name: 'CloudSyncNotifier',
        error: e,
      );
      state = state.copyWith(
        isSyncing: false,
        lastError: 'Authentication failed: $e',
      );
    }
  }

  /// Sign out from the cloud provider.
  Future<void> signOut() async {
    state = const CloudSyncState();
    developer.log(
      'Signed out from cloud provider',
      name: 'CloudSyncNotifier',
    );
  }

  /// Set the cloud provider.
  void setProvider(String providerName) {
    state = state.copyWith(
      providerName: providerName,
      isAuthenticated: false,
      accountName: null,
    );
  }

  /// Sync a single project.
  ///
  /// Returns `true` if the sync succeeded, `false` otherwise.
  Future<bool> syncProject(String projectId) async {
    if (state.isSyncing) return false;

    state = state.copyWith(isSyncing: true, clearError: true);

    try {
      final api = EngineService.instance.api;
      final result = await api.syncProject(projectId: projectId);

      state = state.copyWith(
        isSyncing: false,
        lastError: result.success ? null : result.message,
      );

      return result.success;
    } catch (e) {
      developer.log(
        'Sync project failed: $e',
        name: 'CloudSyncNotifier',
        error: e,
      );
      state = state.copyWith(
        isSyncing: false,
        lastError: 'Sync failed: $e',
      );
      return false;
    }
  }

  /// Get the sync status for a project.
  Future<Map<String, dynamic>?> getSyncStatus(String projectId) async {
    try {
      final api = EngineService.instance.api;
      final status = await api.getSyncStatus(projectId: projectId);
      return {
        'projectId': status.projectId,
        'status': status.status,
        'statusDisplayName': status.statusDisplayName,
        'isActionable': status.isActionable,
        'lastSyncedAt': status.lastSyncedAt,
        'errorMessage': status.errorMessage,
      };
    } catch (e) {
      developer.log(
        'Get sync status failed: $e',
        name: 'CloudSyncNotifier',
        error: e,
      );
      return null;
    }
  }

  /// Fetch the list of cloud projects.
  Future<void> fetchCloudProjects() async {
    try {
      final api = EngineService.instance.api;
      final projects = await api.getCloudProjects();

      state = state.copyWith(
        cloudProjects: projects
            .map((p) => CloudProjectEntry(
                  projectId: p.projectId,
                  name: p.name,
                  modifiedAt: p.modifiedAt,
                  sizeBytes: p.sizeBytes,
                  cloudFileId: p.cloudFileId,
                  providerName: p.providerName,
                ))
            .toList(),
      );
    } catch (e) {
      developer.log(
        'Fetch cloud projects failed: $e',
        name: 'CloudSyncNotifier',
        error: e,
      );
      // Keep the existing project list on error
    }
  }

  /// Resolve a sync conflict for a project.
  ///
  /// `strategy` must be one of: "KeepLocal", "KeepCloud", "KeepBoth",
  /// or "AutoMerge".
  Future<bool> resolveConflict(String projectId, String strategy) async {
    try {
      final api = EngineService.instance.api;
      await api.resolveSyncConflict(projectId: projectId, strategy: strategy);

      // Update conflict count
      final newCount = state.pendingConflicts > 0
          ? state.pendingConflicts - 1
          : 0;
      state = state.copyWith(pendingConflicts: newCount);
      return true;
    } catch (e) {
      developer.log(
        'Resolve conflict failed: $e',
        name: 'CloudSyncNotifier',
        error: e,
      );
      state = state.copyWith(lastError: 'Conflict resolution failed: $e');
      return false;
    }
  }

  /// Clear the last error message.
  void clearError() {
    state = state.copyWith(clearError: true);
  }
}

// ─── Provider ─────────────────────────────────────────────────────

/// Provider that exposes [CloudSyncNotifier] and the current [CloudSyncState].
final cloudSyncProvider =
    StateNotifierProvider<CloudSyncNotifier, CloudSyncState>((ref) {
  return CloudSyncNotifier();
});
