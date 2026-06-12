plugins {
    id("com.android.application")
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "com.editorspro.editors_pro"
    compileSdk = 35
    ndkVersion = "27.0.12077973"

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    // Release signing configuration.
    // For Play Store distribution, set these environment variables or add them to local.properties:
    //   EDITORS_PRO_STORE_FILE=/path/to/keystore.jks
    //   EDITORS_PRO_KEY_ALIAS=upload
    //   EDITORS_PRO_STORE_PASSWORD=***
    //   EDITORS_PRO_KEY_PASSWORD=***
    // Never commit keystore files or passwords to version control.
    signingConfigs {
        create("release") {
            val storeFilePath = System.getenv("EDITORS_PRO_STORE_FILE")
                ?: findProperty("EDITORS_PRO_STORE_FILE") as? String
            val storePassword = System.getenv("EDITORS_PRO_STORE_PASSWORD")
                ?: findProperty("EDITORS_PRO_STORE_PASSWORD") as? String
            val keyAlias = System.getenv("EDITORS_PRO_KEY_ALIAS")
                ?: findProperty("EDITORS_PRO_KEY_ALIAS") as? String
            val keyPassword = System.getenv("EDITORS_PRO_KEY_PASSWORD")
                ?: findProperty("EDITORS_PRO_KEY_PASSWORD") as? String

            if (storeFilePath != null && storePassword != null && keyAlias != null && keyPassword != null) {
                storeFile = file(storeFilePath)
                this.storePassword = storePassword
                this.keyAlias = keyAlias
                this.keyPassword = keyPassword
            }
        }
    }

    defaultConfig {
        applicationId = "com.editorspro.editors_pro"
        // Video editing requires higher minSdk for MediaCodec and hardware acceleration
        minSdk = 24
        targetSdk = 35
        versionCode = flutter.versionCode
        versionName = flutter.versionName

        // Support for all common ABIs
        ndk {
            abiFilters += listOf("armeabi-v7a", "arm64-v8a", "x86_64")
        }

        // Rust library loading
        sourceSets {
            getByName("main") {
                jniLibs.srcDirs("src/main/jniLibs")
            }
        }
    }

    buildTypes {
        debug {
            isMinifyEnabled = false
            isShrinkResources = false
            // Enable debug native symbols
            ndk {
                debugSymbolLevel = "FULL"
            }
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            // Use the release signing config if keystore is configured,
            // otherwise fall back to debug signing for development builds.
            signingConfig = if (signingConfigs.findByName("release")?.storeFile != null) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }

    // Lint options for video editor
    lint {
        disable += "MissingTranslation"
        abortOnError = false
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

dependencies {
    // ExoPlayer for video playback (fallback)
    implementation("com.google.android.exoplayer:exoplayer:2.19.1")
    implementation("com.google.android.exoplayer:exoplayer-ui:2.19.1")
}

flutter {
    source = "../.."
}
