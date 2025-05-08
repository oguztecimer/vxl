fn main() {
    // Set DYLD_LIBRARY_PATH for the build process
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    println!("cargo:rustc-env=DYLD_LIBRARY_PATH=/Users/oguztecimer/VulkanSDK/1.4.313.0/macOS/lib");
}
