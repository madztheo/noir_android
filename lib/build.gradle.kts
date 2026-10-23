plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
    id("de.undercouch.download") version "5.6.0"
}

android {
    namespace = "com.noirandroid.lib"
    compileSdk = 34

    defaultConfig {
        minSdk = 24

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
}

dependencies {

    implementation("androidx.core:core-ktx:1.9.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.8.0")
    implementation("com.google.code.gson:gson:2.8.9")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                groupId = "com.github.madztheo"
                artifactId = "noir_android"
                version = "v1.0.0-rc.3-1"
            }
        }
    }
}

val rustLibName = "noir_java" // Adjust based on your library name
val rustLibPath = "src/main/java/$rustLibName" // Adjust based on your library name
val releaseUrl = "https://github.com/madztheo/noir_android/releases/download/v1.0.0-rc.3-1"

// Find NDK toolchain bin directory to make cross-compilation tools available to cargo
val ndkToolchainBin: String by lazy {
    val sdkDir = android.sdkDirectory
    val ndkDir = file("$sdkDir/ndk").listFiles()
        ?.filter { it.isDirectory }
        ?.maxByOrNull { it.name }
        ?: error("No Android NDK found in $sdkDir/ndk")
    val hostTag = if (System.getProperty("os.name").lowercase().contains("mac")) "darwin-x86_64" else "linux-x86_64"
    "$ndkDir/toolchains/llvm/prebuilt/$hostTag/bin"
}

// Download libc++.so (standard LLVM libc++ with std::__1 namespace) for each ABI.
// The pre-built barretenberg uses standard LLVM libc++ (not the NDK's __ndk1
// variant), so we ship it as libc++.so (matching its SONAME) to avoid conflicts
// with the NDK's libc++_shared.so in consuming apps. It is also needed at link
// time: the NDK sysroot has no libc++.so, so -lc++ (emitted by barretenberg-rs)
// only resolves through the link search path added in buildRust, giving the
// final binary NEEDED:libc++.so instead of NEEDED:libc++_shared.so.
tasks.register("fetchLibCpp") {
    doLast {
        download.run {
            src("$releaseUrl/libc++_arm64-v8a.so")
            dest("src/main/jniLibs/arm64-v8a/libc++.so")
            overwrite(false)
        }
        download.run {
            src("$releaseUrl/libc++_x86_64.so")
            dest("src/main/jniLibs/x86_64/libc++.so")
            overwrite(false)
        }
    }
}

tasks.register("buildRust") {
    dependsOn("fetchLibCpp")
    doLast {
        val pathWithNdk = "$ndkToolchainBin:${System.getenv("PATH")}"
        // The barretenberg static library (libbb-external.a) is downloaded by
        // barretenberg-rs' build.rs. The jniLibs dir is passed as a link search
        // path so -lc++ resolves to the libc++.so we ship (see fetchLibCpp).
        // Android arm64
        exec {
            workingDir(file(rustLibPath))
            environment("PATH", pathWithNdk)
            commandLine(
                "cargo", "rustc", "--release", "--target", "aarch64-linux-android",
                "--", "-L", "native=${file("src/main/jniLibs/arm64-v8a").absolutePath}"
            )
        }
        // Android x86_64
        exec {
            workingDir(file(rustLibPath))
            environment("PATH", pathWithNdk)
            commandLine(
                "cargo", "rustc", "--release", "--target", "x86_64-linux-android",
                "--", "-L", "native=${file("src/main/jniLibs/x86_64").absolutePath}"
            )
        }
    }
}

tasks.register("copyRustLibs") {
    dependsOn("fetchLibCpp")
    if (System.getenv("BUILD_TYPE") == "MANUAL") {
        dependsOn("buildRust")
    }
    doLast {
        if (System.getenv("BUILD_TYPE") == "MANUAL") {
            // Copy the compiled library (.so file) to the appropriate JNI folder
            copy {
                from("$rustLibPath/target/aarch64-linux-android/release")
                include("lib${rustLibName}.so")
                into("src/main/jniLibs/arm64-v8a")
            }
            copy {
                from("$rustLibPath/target/x86_64-linux-android/release")
                include("lib${rustLibName}.so")
                into("src/main/jniLibs/x86_64")
            }
        } else {
            // Download the .so files from the GitHub release
            download.run {
                src("$releaseUrl/libnoir_java_arm64-v8a.so")
                dest("src/main/jniLibs/arm64-v8a/libnoir_java.so")
                overwrite(false)
            }
            download.run {
                src("$releaseUrl/libnoir_java_x86_64.so")
                dest("src/main/jniLibs/x86_64/libnoir_java.so")
                overwrite(false)
            }
        }
    }
}

tasks.whenTaskAdded {
    if (name.matches(Regex("merge.*JniLibFolders"))) {
        dependsOn("copyRustLibs")
    }
}
