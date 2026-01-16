#!/bin/bash
cargo ndk -t arm64-v8a -- build --release
adb push target/aarch64-linux-android/release/libnapoleon_amp_android.so /data/local/tmp/
adb shell "export LD_LIBRARY_PATH=/data/local/tmp && /data/local/tmp/your_binary"
