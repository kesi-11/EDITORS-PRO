package com.editorspro.editors_pro

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.plugin.common.MethodChannel
import java.nio.ByteBuffer
import java.nio.ByteOrder

class MainActivity : FlutterActivity() {

    companion object {
        private const val EXPORT_CHANNEL = "com.editorspro/editors_pro/export"
        private const val AUDIO_CHANNEL = "com.editorspro/audio"
    }

    private var audioTrack: AudioTrack? = null
    private var sampleRate = 44100
    private var channels = 2

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

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
                                // Convert ByteArray to FloatArray
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
}
