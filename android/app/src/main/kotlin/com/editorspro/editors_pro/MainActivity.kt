package com.editorspro.editors_pro

import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {

    companion object {
        private const val CHANNEL = "com.editorspro.editors_pro/export"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Set up method channel for export foreground service
        MethodChannel(flutterEngine?.dartExecutor?.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "startExport" -> {
                        ExportService.start(this)
                        result.success(null)
                    }
                    "updateProgress" -> {
                        val progress = call.argument<Int>("progress") ?: 0
                        val stage = call.argument<String>("stage") ?: "Encoding"
                        ExportService.updateProgress(this, progress, stage)
                        result.success(null)
                    }
                    "complete" -> {
                        val filePath = call.argument<String>("filePath") ?: ""
                        val fileSize = call.argument<String>("fileSize") ?: ""
                        ExportService.complete(this, filePath, fileSize)
                        result.success(null)
                    }
                    "cancel" -> {
                        ExportService.cancel(this)
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }
    }
}
