fn main() {
    println!("cargo:rerun-if-changed=cpp/ceres_bridge.cpp");
    println!("cargo:rerun-if-changed=cpp/ceres_bridge.h");

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("cpp/ceres_bridge.cpp")
        .include("cpp")
        .include("/usr/include/eigen3")
        .compile("ceres_bridge");

    println!("cargo:rustc-link-lib=ceres");
    println!("cargo:rustc-link-lib=glog");
    println!("cargo:rustc-link-lib=gflags");
}
