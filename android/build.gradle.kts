allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)
}
subprojects {
    project.evaluationDependsOn(":app")
}
subprojects {
    plugins.withId("com.android.library") {
        if (name == "file_picker") {
            plugins.apply("org.jetbrains.kotlin.android")
        }
        extensions.configure<com.android.build.gradle.LibraryExtension>("android") {
            compileSdk = 36
            ndkVersion = "28.2.13676358"
        }
    }
    // Force compileSdk for all subprojects that have an android extension
    val forceCompileSdk = Action<Project> {
        if (hasProperty("android")) {
            val android = extensions.getByName("android")
            if (android is com.android.build.gradle.BaseExtension) {
                android.compileSdkVersion(36)
            }
        }
    }
    if (state.executed) {
        forceCompileSdk.execute(this)
    } else {
        afterEvaluate { forceCompileSdk.execute(this) }
    }
    plugins.withId("org.jetbrains.kotlin.android") {
        extensions.configure<org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension>("kotlin") {
            compilerOptions {
                jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
            }
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
