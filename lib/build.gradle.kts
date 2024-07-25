plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
    id("de.undercouch.download") version "5.6.0"
}

android {
    namespace = "com.noirandroid.lib"
    compileSdk = 33

    defaultConfig {
        minSdk = 23

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
                version = "v0.30.0-9"
            }
        }
    }
}

val rustLibName = "noir_java" // Adjust based on your library name
val rustLibPath = "src/main/java/$rustLibName" // Adjust based on your library name

tasks.register<Exec>("buildRust") {
    workingDir(file(rustLibPath))
    // Set the environment variables necessary for OpenSSL
    val androidHome = System.getenv("ANDROID_HOME")
    val ndkVersion = System.getenv("NDK_VERSION")
    val hostTag = System.getenv("HOST_TAG")
    val path = System.getenv("PATH")
    val androidNdkHome = "$androidHome/ndk/$ndkVersion"
    val toolchain = "$androidNdkHome/toolchains/llvm/prebuilt/$hostTag"
    val target = "aarch64-linux-android"
    val api = "33"
    val cc = "$toolchain/bin/$target$api-clang"

    environment("ANDROID_NDK_HOME", androidNdkHome)
    environment("TOOLCHAIN", toolchain)
    environment("TARGET", target)
    environment("API", api)
    environment("AR", "$toolchain/bin/llvm-ar")
    environment("CC", cc)
    environment("AS", cc)
    environment("CXX", "$toolchain/bin/$target$api-clang++")
    environment("LD", "$toolchain/bin/ld")
    environment("RANLIB", "$toolchain/bin/llvm-ranlib")
    environment("STRIP", "$toolchain/bin/llvm-strip")
    environment("PATH", "$path:$androidHome/cmdline-tools/latest/bin")
    environment("PATH", "$path:$toolchain/bin")
    environment("CMAKE_TOOLCHAIN_FILE_aarch64_linux_android", "$androidNdkHome/build/cmake/android.toolchain.cmake")
    environment("ANDROID_ABI", "arm64-v8a")
    // Android arm64
    commandLine("cargo", "build", "--release", "--target", "aarch64-linux-android", "-vvvv")
    // Android arm
    // commandLine("cargo", "build", "--release", "--target", "armv7-linux-androideabi")
    // Android x86
    // commandLine("cargo", "build", "--release", "--target", "i686-linux-android")
    // Android x86_64
    // commandLine("cargo", "build", "--release", "--target", "x86_64-linux-android")
}

tasks.register<Copy>("copyRustLibs") {
    val buildType = System.getenv("BUILD_TYPE")
    if (buildType == "MANUAL") {
        // Copy the compiled library (.so file) to the appropriate JNI folder
        from("$rustLibPath/target/aarch64-linux-android/release")
        into("src/main/jniLibs/arm64-v8a")
    } else {
        // Download the .so file from the GitHub release
        download.run {
            src("https://github.com/madztheo/noir_android/releases/download/v0.30.0-9/libnoir_java_arm64-v8a.so")
            dest("src/main/jniLibs/arm64-v8a/libnoir_java.so")
            overwrite(false)
        }
    }
    include("lib${rustLibName}.so")
}

tasks.whenTaskAdded {
    val buildType = System.getenv("BUILD_TYPE")
    if (buildType == "MANUAL") {
        if (name == "javaPreCompileDebug" || name == "javaPreCompileRelease") {
            dependsOn("buildRust")
        }
    }
    if (name.matches(Regex("merge.*JniLibFolders"))) {
        dependsOn("copyRustLibs")
    }
}
