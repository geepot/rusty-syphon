use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "macos" {
        build_syphon();
        build_spout_stub_macos();
    } else if target_os == "windows" {
        build_spout();
    } else {
        println!("cargo:warning=rusty-syphon-spout: Syphon (macOS) and Spout (Windows) only; skipping native build");
    }
}

fn framework_supports_target_arch(framework: &PathBuf, target_arch: &str) -> bool {
    let bin = framework.join("Syphon");
    if !bin.exists() {
        return false;
    }
    let output = Command::new("lipo").arg("-archs").arg(&bin).output();
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let arches = String::from_utf8_lossy(&output.stdout);
    arches.split_whitespace().any(|a| a == target_arch)
}

fn build_syphon() {
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
    stage_syphon_framework_for_runtime(&syphon_framework_dir);

    // Re-run if these change
    println!("cargo:rerun-if-changed=syphon_glue/syphon_glue.h");
    println!("cargo:rerun-if-changed=syphon_glue/syphon_glue.m");
    println!("cargo:rerun-if-env-changed=SYPHON_FRAMEWORK_PATH");
}

fn stage_syphon_framework_for_runtime(syphon_framework_dir: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // OUT_DIR .../<profile>/build/<pkg>/out -> profile dir is 3 levels up from OUT_DIR
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };
    let frameworks_dir = profile_dir.join("Frameworks");
    let dst = frameworks_dir.join("Syphon.framework");
    if let Err(e) = copy_dir_recursive(syphon_framework_dir, &dst) {
        println!(
            "cargo:warning=Failed to stage Syphon.framework to {}: {}",
            dst.display(),
            e
        );
    }
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::copy(&from, &to)?;
        } else if ty.is_symlink() {
            // Resolve symlink target contents for portability in the staged bundle.
            let target = fs::read_link(&from)?;
            let resolved = if target.is_absolute() {
                target
            } else {
                from.parent().unwrap_or(src).join(target)
            };
            if resolved.is_dir() {
                copy_dir_recursive(&resolved, &to)?;
            } else if resolved.is_file() {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)?;
                }
                let _ = fs::copy(&resolved, &to)?;
            }
        }
    }
    Ok(())
}

/// Resolve lib/bin dirs from a candidate root: accept root, root/lib, root/bin (grabs what it needs).
fn resolve_spout_prebuilt(root: &PathBuf) -> Option<(PathBuf, PathBuf)> {
    let lib_dir = if root.join("lib").join("SpoutLibrary.lib").exists() {
        root.join("lib")
    } else if root.join("SpoutLibrary.lib").exists() {
        root.clone()
    } else {
        return None;
    };
    let bin_dir = if root.join("bin").join("SpoutLibrary.dll").exists() {
        root.join("bin")
    } else if root.join("SpoutLibrary.dll").exists() {
        root.clone()
    } else {
        root.join("bin") // copy_dll will no-op if dll not present
    };
    Some((lib_dir, bin_dir))
}

fn build_spout() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prebuilt_root = manifest_dir.join("prebuilt").join("windows");

    // Prefer prebuilt: check prebuilt/windows, prebuilt/windows/MD, prebuilt/windows/MT (each can be flat or have lib/ and bin/)
    let prebuilt = [prebuilt_root.clone(), prebuilt_root.join("MD"), prebuilt_root.join("MT")]
        .into_iter()
        .find_map(|root| resolve_spout_prebuilt(&root));

    let (spout_lib, spout_bin) = if let Some((lib_dir, bin_dir)) = prebuilt {
        println!("cargo:warning=Using prebuilt Spout (lib from {}, bin from {})", lib_dir.display(), bin_dir.display());
        println!("cargo:rerun-if-changed=prebuilt/windows");
        (lib_dir, bin_dir)
    } else {
        let dst = cmake::Config::new("Spout2")
            .define("SPOUT_BUILD_SPOUTDX", "OFF")
            .define("SPOUT_BUILD_LIBRARY", "ON")
            .build();
        let spout_lib = dst.join("lib");
        let spout_bin = dst.join("bin");
        if !spout_lib.exists() {
            let build_lib = dst.join("build").join("lib");
            let build_bin = dst.join("build").join("bin");
            if build_lib.exists() {
                (build_lib, build_bin)
            } else {
                panic!(
                    "Spout2 build did not produce lib at {} or {}",
                    spout_lib.display(),
                    dst.join("build").join("lib").display()
                );
            }
        } else {
            (spout_lib, spout_bin)
        }
    };

    println!("cargo:rustc-link-search=native={}", spout_lib.display());
    copy_dll(&spout_bin, "SpoutLibrary.dll");
    println!("cargo:rustc-link-lib=SpoutLibrary");

    // Compile spout_glue.cpp (includes SpoutLibrary.h, links to SpoutLibrary)
    let spout_include = manifest_dir.join("Spout2").join("SPOUTSDK").join("SpoutLibrary");
    cc::Build::new()
        .cpp(true)
        .file("spout_glue/spout_glue.cpp")
        .include(&spout_include)
        .include("spout_glue")
        .compile("spout_glue");

    // Generate Rust bindings for spout_glue.h
    let bindings = bindgen::Builder::default()
        .header("spout_glue/spout_glue.h")
        .allowlist_function("spout_.*")
        .allowlist_type("spout_handle")
        .generate()
        .expect("Failed to generate Spout bindings");

    bindings
        .write_to_file(out_dir.join("spout_bindings.rs"))
        .expect("Failed to write spout_bindings.rs");

    println!("cargo:rerun-if-changed=spout_glue/spout_glue.h");
    println!("cargo:rerun-if-changed=spout_glue/spout_glue.cpp");
    println!("cargo:rerun-if-changed=Spout2");
}

/// On macOS, Spout2 does not build (Windows-only). Build the stub glue and generate
/// Spout bindings so both code paths are compiled and bindgen is exercised.
fn build_spout_stub_macos() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file("spout_glue/spout_glue_stub.c")
        .include("spout_glue")
        .compile("spout_glue");

    let bindings = bindgen::Builder::default()
        .header("spout_glue/spout_glue.h")
        .allowlist_function("spout_.*")
        .allowlist_type("spout_handle")
        .generate()
        .expect("Failed to generate Spout bindings (stub)");

    bindings
        .write_to_file(out_dir.join("spout_bindings.rs"))
        .expect("Failed to write spout_bindings.rs");

    println!("cargo:rerun-if-changed=spout_glue/spout_glue.h");
    println!("cargo:rerun-if-changed=spout_glue/spout_glue_stub.c");
}

fn copy_dll(bin_dir: &PathBuf, dll_name: &str) {
    let src = bin_dir.join(dll_name);
    if !src.exists() {
        return;
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let profile_dir = if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target_dir).join(&profile)
    } else {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        // OUT_DIR .../<profile>/build/<pkg>/out
        out_dir
            .ancestors()
            .nth(3)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join(&profile))
    };

    for dest_dir in [
        profile_dir.clone(),
        profile_dir.join("deps"),
        profile_dir.join("examples"),
    ] {
        let _ = fs::create_dir_all(&dest_dir);
        let _ = fs::copy(&src, dest_dir.join(dll_name));
    }
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
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".to_string());

    if let Ok(path) = env::var("SYPHON_FRAMEWORK_PATH") {
        let p = PathBuf::from(&path);
        let framework = p.join("Syphon.framework");
        if framework.exists() {
            if !framework_supports_target_arch(&framework, &target_arch) {
                panic!(
                    "SYPHON_FRAMEWORK_PATH points to Syphon.framework without target arch '{}' at {}",
                    target_arch,
                    framework.display()
                );
            }
            println!("cargo:rerun-if-changed=ignore"); // env already triggers rerun
            return framework;
        }
        // Path might be the framework dir itself
        if p.ends_with("Syphon.framework") && p.exists() {
            if !framework_supports_target_arch(&p, &target_arch) {
                panic!(
                    "SYPHON_FRAMEWORK_PATH points to Syphon.framework without target arch '{}' at {}",
                    target_arch,
                    p.display()
                );
            }
            return p;
        }
        panic!(
            "SYPHON_FRAMEWORK_PATH set but Syphon.framework not found at {}",
            path
        );
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Check prebuilt/macos/ first so we don't build from source if user dropped a framework
    let prebuilt_framework = manifest_dir.join("prebuilt").join("macos").join("Syphon.framework");
    if prebuilt_framework.exists() {
        if framework_supports_target_arch(&prebuilt_framework, &target_arch) {
            println!("cargo:rerun-if-changed=prebuilt/macos");
            return prebuilt_framework;
        }
        println!(
            "cargo:warning=Ignoring prebuilt Syphon.framework at {} because it does not contain target arch '{}'",
            prebuilt_framework.display(),
            target_arch
        );
    }

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
