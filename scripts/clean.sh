./gradlew clean
cd lib/src/main/java/noir_java && cargo clean
rm -rf Cargo.lock
cd ../../../../../
rm -rf lib/src/main/jniLibs/arm64-v8a/libnoir_java.so