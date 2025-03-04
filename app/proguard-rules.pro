# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# Keep JNI methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep the application class
-keep class com.noirandroid.testapp.** { *; }
-keep class com.noirandroid.lib.** { *; } 