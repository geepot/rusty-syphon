use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "macos" {
        println!("cargo:warning=rusty-syphon is macOS-only; skipping Syphon build");
        return;
    }

    let syphon_framework_dir = find_or_build_syphon_framework();
    let framework_parent = syphon_framework_dir
        .parent()
        .expect("Syphon.framework has parent");
    let sdk_path = sdk_path();

    // Compile the C/ObjC glue with ARC so __bridge_retained/__bridge_transfer work (no warnings)
    let mut cc = cc::Build::new();
    cc.file("syphon_glue/syphon_glue.m")
        .include("syphon_glue")
        .flag("-fobjc-arc")
        .flag("-F")
        .flag(framework_parent.to_str().unwrap())
        .flag("-isysroot")
        .flag(&sdk_path)
        .compile("syphon_glue");

    // Run bindgen on the glue header
    let bindings = bindgen::Builder::default()
        .header("syphon_glue/syphon_glue.h")
        .clang_arg("-F")
        .clang_arg(framework_parent.to_str().unwrap())
        .clang_arg("-isysroot")
        .clang_arg(&sdk_path)
        .allowlist_function("syphon_.*")
        .generate()
        .expect("Failed to generate bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings");

    // Link frameworks (Rust binary links these; the glue .a has undefined refs to Syphon/Foundation/etc.)
    println!("cargo:rustc-link-search=framework={}", framework_parent.display());
    // So the binary finds Syphon.framework at runtime (dyld @rpath)
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        framework_parent.display()
    );
    println!("cargo:rustc-link-lib=framework=Syphon");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=OpenGL");
    println!("cargo:rustc-link-lib=framework=IOSurface");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
    println!("cargo:rustc-link-lib=framework=AppKit");

    // Re-run if these change
    println!("cargo:rerun-if-changed=syphon_glue/syphon_glue.h");
    println!("cargo:rerun-if-changed=syphon_glue/syphon_glue.m");
    println!("cargo:rerun-if-env-changed=SYPHON_FRAMEWORK_PATH");
}

fn sdk_path() -> String {
    let output = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-path"])
        .output()
        .expect("xcrun --sdk macosx --show-sdk-path failed");
    assert!(output.status.success(), "xcrun failed: {:?}", output);
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn find_or_build_syphon_framework() -> PathBuf {
    if let Ok(path) = env::var("SYPHON_FRAMEWORK_PATH") {
        let p = PathBuf::from(&path);
        let framework = p.join("Syphon.framework");
        if framework.exists() {
            println!("cargo:rerun-if-changed=ignore"); // env already triggers rerun
            return framework;
        }
        // Path might be the framework dir itself
        if p.ends_with("Syphon.framework") && p.exists() {
            return p;
        }
        panic!(
            "SYPHON_FRAMEWORK_PATH set but Syphon.framework not found at {}",
            path
        );
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let derived_data = manifest_dir.join("target").join("syphon-build");
    let framework = derived_data
        .join("Build")
        .join("Products")
        .join("Release")
        .join("Syphon.framework");

    if framework.exists() {
        println!("cargo:rerun-if-changed=Syphon-Framework");
        return framework;
    }

    println!("cargo:warning=Building Syphon.framework with xcodebuild (run 'xcodebuild -downloadComponent MetalToolchain' if Metal compile fails)");
    let status = Command::new("xcodebuild")
        .args([
            "-project",
            "Syphon-Framework/Syphon.xcodeproj",
            "-scheme",
            "Syphon",
            "-configuration",
            "Release",
            "-derivedDataPath",
            derived_data.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run xcodebuild");

    if !status.success() {
        panic!(
            "xcodebuild failed. If the error mentions Metal, run: xcodebuild -downloadComponent MetalToolchain"
        );
    }

    if !framework.exists() {
        panic!(
            "xcodebuild succeeded but Syphon.framework not found at {}",
            framework.display()
        );
    }

    println!("cargo:rerun-if-changed=Syphon-Framework");
    framework
}
