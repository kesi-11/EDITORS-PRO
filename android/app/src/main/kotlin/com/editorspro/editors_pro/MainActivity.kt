package com.editorspro.editors_pro

import android.Manifest
import android.content.pm.PackageManager
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.os.Bundle
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.flutter.embedding.android.FlutterActivity
import io.flutter.plugin.common.MethodChannel
import java.nio.ByteBuffer
import java.nio.ByteOrder

class MainActivity : FlutterActivity() {

    companion object {
        private const val TAG = "EditorsPro"
        private const val EXPORT_CHANNEL = "com.editorspro/editors_pro/export"
        private const val AUDIO_CHANNEL = "com.editorspro/audio"
        private const val PERMISSION_REQUEST_CODE = 1001

        // Permissions required for Android 13+
        private val REQUIRED_PERMISSIONS = arrayOf(
            Manifest.permission.READ_MEDIA_VIDEO,
            Manifest.permission.READ_MEDIA_AUDIO,
            Manifest.permission.READ_MEDIA_IMAGES,
            Manifest.permission.POST_NOTIFICATIONS,
        )

        // Legacy permissions for Android < 13
        private val LEGACY_PERMISSIONS = arrayOf(
            Manifest.permission.READ_EXTERNAL_STORAGE,
            Manifest.permission.WRITE_EXTERNAL_STORAGE,
        )
    }

    private var audioTrack: AudioTrack? = null
    private var sampleRate = 44100
    private var channels = 2
    private var pendingPermissionResult: MethodChannel.Result? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Request runtime permissions on startup
        requestRequiredPermissions()

        // Set up method channel for export foreground service
        MethodChannel(flutterEngine?.dartExecutor?.binaryMessenger, EXPORT_CHANNEL)
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
                    "requestPermissions" -> {
                        requestRequiredPermissions()
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }

        // Set up method channel for audio playback
        MethodChannel(flutterEngine?.dartExecutor?.binaryMessenger, AUDIO_CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "initialize" -> {
                        try {
                            sampleRate = call.argument<Int>("sampleRate") ?: 44100
                            channels = call.argument<Int>("channels") ?: 2

                            val channelConfig = if (channels == 1)
                                AudioFormat.CHANNEL_OUT_MONO
                            else
                                AudioFormat.CHANNEL_OUT_STEREO

                            val bufferSize = AudioTrack.getMinBufferSize(
                                sampleRate,
                                channelConfig,
                                AudioFormat.ENCODING_PCM_FLOAT
                            )

                            audioTrack = AudioTrack.Builder()
                                .setAudioAttributes(
                                    AudioAttributes.Builder()
                                        .setUsage(AudioAttributes.USAGE_MEDIA)
                                        .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                                        .build()
                                )
                                .setAudioFormat(
                                    AudioFormat.Builder()
                                        .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                                        .setSampleRate(sampleRate)
                                        .setChannelMask(channelConfig)
                                        .build()
                                )
                                .setBufferSizeInBytes(bufferSize)
                                .setTransferMode(AudioTrack.MODE_STREAM)
                                .build()

                            result.success(true)
                        } catch (e: Exception) {
                            result.error("INIT_ERROR", e.message, null)
                        }
                    }

                    "writeSamples" -> {
                        try {
                            val bytes = call.argument<ByteArray>("samples")
                            if (bytes != null && audioTrack != null) {
                                val floatBuffer = ByteBuffer.wrap(bytes)
                                    .order(ByteOrder.LITTLE_ENDIAN)
                                val floatArray = FloatArray(bytes.size / 4)
                                floatBuffer.asFloatBuffer().get(floatArray)

                                audioTrack?.write(floatArray, 0, floatArray.size, AudioTrack.WRITE_NON_BLOCKING)
                            }
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("WRITE_ERROR", e.message, null)
                        }
                    }

                    "play" -> {
                        try {
                            audioTrack?.play()
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("PLAY_ERROR", e.message, null)
                        }
                    }

                    "pause" -> {
                        try {
                            audioTrack?.pause()
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("PAUSE_ERROR", e.message, null)
                        }
                    }

                    "stop" -> {
                        try {
                            audioTrack?.stop()
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("STOP_ERROR", e.message, null)
                        }
                    }

                    "seekTo" -> {
                        try {
                            audioTrack?.stop()
                            audioTrack?.flush()
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("SEEK_ERROR", e.message, null)
                        }
                    }

                    "setVolume" -> {
                        try {
                            val volume = call.argument<Double>("volume")?.toFloat() ?: 1.0f
                            audioTrack?.setVolume(volume.coerceIn(0.0f, 1.0f))
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("VOLUME_ERROR", e.message, null)
                        }
                    }

                    "release" -> {
                        try {
                            audioTrack?.stop()
                            audioTrack?.release()
                            audioTrack = null
                            result.success(null)
                        } catch (e: Exception) {
                            result.error("RELEASE_ERROR", e.message, null)
                        }
                    }

                    else -> result.notImplemented()
                }
            }
    }

    override fun onResume() {
        super.onResume()
        // Notify the engine that the activity is resuming
        // (for future: engine.pause() / engine.resume() lifecycle hooks)
    }

    override fun onPause() {
        super.onPause()
        // Notify the engine that the activity is pausing
    }

    override fun onDestroy() {
        super.onDestroy()
        try {
            audioTrack?.stop()
            audioTrack?.release()
            audioTrack = null
        } catch (_: Exception) {
            // Ignore cleanup errors
        }
    }

    // ─── Permission Handling ────────────────────────────────────────

    /**
     * Request runtime permissions based on Android version.
     * - Android 13+ (API 33+): READ_MEDIA_VIDEO, READ_MEDIA_AUDIO, READ_MEDIA_IMAGES, POST_NOTIFICATIONS
     * - Android < 13: READ_EXTERNAL_STORAGE, WRITE_EXTERNAL_STORAGE
     */
    private fun requestRequiredPermissions() {
        val permissions = if (android.os.Build.VERSION.SDK_INT >= 33) {
            REQUIRED_PERMISSIONS
        } else {
            LEGACY_PERMISSIONS
        }

        val ungranted = permissions.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }.toTypedArray()

        if (ungranted.isNotEmpty()) {
            Log.w(TAG, "Requesting permissions: ${ungranted.joinToString()}")
            ActivityCompat.requestPermissions(this, ungranted, PERMISSION_REQUEST_CODE)
        } else {
            Log.i(TAG, "All required permissions already granted")
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == PERMISSION_REQUEST_CODE) {
            val denied = permissions.zip(grantResults.toTypedArray())
                .filter { it.second != PackageManager.PERMISSION_GRANTED }
            if (denied.isNotEmpty()) {
                Log.w(TAG, "Permissions denied: ${denied.joinToString { it.first }}")
            } else {
                Log.i(TAG, "All requested permissions granted")
            }
        }
    }
}
