// GENERATED CODE - DO NOT MODIFY BY HAND
// This is a placeholder for the drift-generated code.
// Run `dart run build_runner build` to generate the actual code.

part of 'app_database.dart';

class ProjectEntry extends DataClass implements Insertable<ProjectEntriesCompanion> {
  final String id;
  final String name;
  final int width;
  final int height;
  final double fps;
  final int durationMs;
  final int createdAt;
  final int updatedAt;
  final String? thumbnailPath;
  final String? eppFilePath;

  ProjectEntry({
    required this.id,
    required this.name,
    required this.width,
    required this.height,
    required this.fps,
    required this.durationMs,
    required this.createdAt,
    required this.updatedAt,
    this.thumbnailPath,
    this.eppFilePath,
  });

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['name'] = Variable<String>(name);
    map['width'] = Variable<int>(width);
    map['height'] = Variable<int>(height);
    map['fps'] = Variable<double>(fps);
    map['duration_ms'] = Variable<int>(durationMs);
    map['created_at'] = Variable<int>(createdAt);
    map['updated_at'] = Variable<int>(updatedAt);
    if (!nullToAbsent || thumbnailPath != null) {
      map['thumbnail_path'] = Variable<String>(thumbnailPath ?? '');
    }
    if (!nullToAbsent || eppFilePath != null) {
      map['epp_file_path'] = Variable<String>(eppFilePath ?? '');
    }
    return map;
  }

  factory ProjectEntry.fromJson(Map<String, dynamic> json, {ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return ProjectEntry(
      id: serializer_.fromJson<String>(json['id']),
      name: serializer_.fromJson<String>(json['name']),
      width: serializer_.fromJson<int>(json['width']),
      height: serializer_.fromJson<int>(json['height']),
      fps: serializer_.fromJson<double>(json['fps']),
      durationMs: serializer_.fromJson<int>(json['durationMs']),
      createdAt: serializer_.fromJson<int>(json['createdAt']),
      updatedAt: serializer_.fromJson<int>(json['updatedAt']),
      thumbnailPath: serializer_.fromJson<String?>(json['thumbnailPath']),
      eppFilePath: serializer_.fromJson<String?>(json['eppFilePath']),
    );
  }

  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer_.toJson<String>(id),
      'name': serializer_.toJson<String>(name),
      'width': serializer_.toJson<int>(width),
      'height': serializer_.toJson<int>(height),
      'fps': serializer_.toJson<double>(fps),
      'durationMs': serializer_.toJson<int>(durationMs),
      'createdAt': serializer_.toJson<int>(createdAt),
      'updatedAt': serializer_.toJson<int>(updatedAt),
      'thumbnailPath': serializer_.toJson<String?>(thumbnailPath),
      'eppFilePath': serializer_.toJson<String?>(eppFilePath),
    };
  }

  ProjectEntriesCompanion toCompanion(bool nullToAbsent) {
    return ProjectEntriesCompanion(
      id: Value(id),
      name: Value(name),
      width: Value(width),
      height: Value(height),
      fps: Value(fps),
      durationMs: Value(durationMs),
      createdAt: Value(createdAt),
      updatedAt: Value(updatedAt),
      thumbnailPath: Value(thumbnailPath),
      eppFilePath: Value(eppFilePath),
    );
  }
}

class ProjectEntriesCompanion extends UpdateCompanion<ProjectEntry> {
  final Value<String> id;
  final Value<String> name;
  final Value<int> width;
  final Value<int> height;
  final Value<double> fps;
  final Value<int> durationMs;
  final Value<int> createdAt;
  final Value<int> updatedAt;
  final Value<String?> thumbnailPath;
  final Value<String?> eppFilePath;

  ProjectEntriesCompanion({
    this.id = const Value.absent(),
    this.name = const Value.absent(),
    this.width = const Value.absent(),
    this.height = const Value.absent(),
    this.fps = const Value.absent(),
    this.durationMs = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.updatedAt = const Value.absent(),
    this.thumbnailPath = const Value.absent(),
    this.eppFilePath = const Value.absent(),
  });

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) map['id'] = Variable<String>(id.value);
    if (name.present) map['name'] = Variable<String>(name.value);
    if (width.present) map['width'] = Variable<int>(width.value);
    if (height.present) map['height'] = Variable<int>(height.value);
    if (fps.present) map['fps'] = Variable<double>(fps.value);
    if (durationMs.present) map['duration_ms'] = Variable<int>(durationMs.value);
    if (createdAt.present) map['created_at'] = Variable<int>(createdAt.value);
    if (updatedAt.present) map['updated_at'] = Variable<int>(updatedAt.value);
    if (thumbnailPath.present) map['thumbnail_path'] = Variable<String>(thumbnailPath.value ?? '');
    if (eppFilePath.present) map['epp_file_path'] = Variable<String>(eppFilePath.value ?? '');
    return map;
  }
}

class MediaAssetEntry extends DataClass implements Insertable<MediaAssetEntriesCompanion> {
  final String id;
  final String projectId;
  final String filePath;
  final String fileName;
  final String mediaType;
  final int? durationMs;
  final int? width;
  final int? height;
  final int fileSizeBytes;
  final String? codec;
  final int? bitrate;
  final String? thumbnailPath;
  final int importedAt;

  MediaAssetEntry({
    required this.id,
    required this.projectId,
    required this.filePath,
    required this.fileName,
    required this.mediaType,
    this.durationMs,
    this.width,
    this.height,
    required this.fileSizeBytes,
    this.codec,
    this.bitrate,
    this.thumbnailPath,
    required this.importedAt,
  });

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['project_id'] = Variable<String>(projectId);
    map['file_path'] = Variable<String>(filePath);
    map['file_name'] = Variable<String>(fileName);
    map['media_type'] = Variable<String>(mediaType);
    if (!nullToAbsent || durationMs != null) map['duration_ms'] = Variable<int>(durationMs ?? 0);
    if (!nullToAbsent || width != null) map['width'] = Variable<int>(width ?? 0);
    if (!nullToAbsent || height != null) map['height'] = Variable<int>(height ?? 0);
    map['file_size_bytes'] = Variable<int>(fileSizeBytes);
    if (!nullToAbsent || codec != null) map['codec'] = Variable<String>(codec ?? '');
    if (!nullToAbsent || bitrate != null) map['bitrate'] = Variable<int>(bitrate ?? 0);
    if (!nullToAbsent || thumbnailPath != null) map['thumbnail_path'] = Variable<String>(thumbnailPath ?? '');
    map['imported_at'] = Variable<int>(importedAt);
    return map;
  }

  factory MediaAssetEntry.fromJson(Map<String, dynamic> json, {ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return MediaAssetEntry(
      id: serializer_.fromJson<String>(json['id']),
      projectId: serializer_.fromJson<String>(json['projectId']),
      filePath: serializer_.fromJson<String>(json['filePath']),
      fileName: serializer_.fromJson<String>(json['fileName']),
      mediaType: serializer_.fromJson<String>(json['mediaType']),
      durationMs: serializer_.fromJson<int?>(json['durationMs']),
      width: serializer_.fromJson<int?>(json['width']),
      height: serializer_.fromJson<int?>(json['height']),
      fileSizeBytes: serializer_.fromJson<int>(json['fileSizeBytes']),
      codec: serializer_.fromJson<String?>(json['codec']),
      bitrate: serializer_.fromJson<int?>(json['bitrate']),
      thumbnailPath: serializer_.fromJson<String?>(json['thumbnailPath']),
      importedAt: serializer_.fromJson<int>(json['importedAt']),
    );
  }

  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer_.toJson<String>(id),
      'projectId': serializer_.toJson<String>(projectId),
      'filePath': serializer_.toJson<String>(filePath),
      'fileName': serializer_.toJson<String>(fileName),
      'mediaType': serializer_.toJson<String>(mediaType),
      'durationMs': serializer_.toJson<int?>(durationMs),
      'width': serializer_.toJson<int?>(width),
      'height': serializer_.toJson<int?>(height),
      'fileSizeBytes': serializer_.toJson<int>(fileSizeBytes),
      'codec': serializer_.toJson<String?>(codec),
      'bitrate': serializer_.toJson<int?>(bitrate),
      'thumbnailPath': serializer_.toJson<String?>(thumbnailPath),
      'importedAt': serializer_.toJson<int>(importedAt),
    };
  }

  MediaAssetEntriesCompanion toCompanion(bool nullToAbsent) {
    return MediaAssetEntriesCompanion(
      id: Value(id),
      projectId: Value(projectId),
      filePath: Value(filePath),
      fileName: Value(fileName),
      mediaType: Value(mediaType),
      durationMs: Value(durationMs),
      width: Value(width),
      height: Value(height),
      fileSizeBytes: Value(fileSizeBytes),
      codec: Value(codec),
      bitrate: Value(bitrate),
      thumbnailPath: Value(thumbnailPath),
      importedAt: Value(importedAt),
    );
  }
}

class MediaAssetEntriesCompanion extends UpdateCompanion<MediaAssetEntry> {
  final Value<String> id;
  final Value<String> projectId;
  final Value<String> filePath;
  final Value<String> fileName;
  final Value<String> mediaType;
  final Value<int?> durationMs;
  final Value<int?> width;
  final Value<int?> height;
  final Value<int> fileSizeBytes;
  final Value<String?> codec;
  final Value<int?> bitrate;
  final Value<String?> thumbnailPath;
  final Value<int> importedAt;

  MediaAssetEntriesCompanion({
    this.id = const Value.absent(),
    this.projectId = const Value.absent(),
    this.filePath = const Value.absent(),
    this.fileName = const Value.absent(),
    this.mediaType = const Value.absent(),
    this.durationMs = const Value.absent(),
    this.width = const Value.absent(),
    this.height = const Value.absent(),
    this.fileSizeBytes = const Value.absent(),
    this.codec = const Value.absent(),
    this.bitrate = const Value.absent(),
    this.thumbnailPath = const Value.absent(),
    this.importedAt = const Value.absent(),
  });

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) map['id'] = Variable<String>(id.value);
    if (projectId.present) map['project_id'] = Variable<String>(projectId.value);
    if (filePath.present) map['file_path'] = Variable<String>(filePath.value);
    if (fileName.present) map['file_name'] = Variable<String>(fileName.value);
    if (mediaType.present) map['media_type'] = Variable<String>(mediaType.value);
    if (durationMs.present) map['duration_ms'] = Variable<int>(durationMs.value ?? 0);
    if (width.present) map['width'] = Variable<int>(width.value ?? 0);
    if (height.present) map['height'] = Variable<int>(height.value ?? 0);
    if (fileSizeBytes.present) map['file_size_bytes'] = Variable<int>(fileSizeBytes.value);
    if (codec.present) map['codec'] = Variable<String>(codec.value ?? '');
    if (bitrate.present) map['bitrate'] = Variable<int>(bitrate.value ?? 0);
    if (thumbnailPath.present) map['thumbnail_path'] = Variable<String>(thumbnailPath.value ?? '');
    if (importedAt.present) map['imported_at'] = Variable<int>(importedAt.value);
    return map;
  }
}

class UserPreference extends DataClass implements Insertable<UserPreferencesCompanion> {
  final String key;
  final String value;

  UserPreference({required this.key, required this.value});

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    return {
      'key': Variable<String>(key),
      'value': Variable<String>(value),
    };
  }

  factory UserPreference.fromJson(Map<String, dynamic> json, {ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return UserPreference(
      key: serializer_.fromJson<String>(json['key']),
      value: serializer_.fromJson<String>(json['value']),
    );
  }

  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    final serializer_ = serializer ?? driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'key': serializer_.toJson<String>(key),
      'value': serializer_.toJson<String>(value),
    };
  }

  UserPreferencesCompanion toCompanion(bool nullToAbsent) {
    return UserPreferencesCompanion(key: Value(key), value: Value(value));
  }
}

class UserPreferencesCompanion extends UpdateCompanion<UserPreference> {
  final Value<String> key;
  final Value<String> value;

  UserPreferencesCompanion({
    this.key = const Value.absent(),
    this.value = const Value.absent(),
  });

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (key.present) map['key'] = Variable<String>(key.value);
    if (value.present) map['value'] = Variable<String>(value.value);
    return map;
  }
}
