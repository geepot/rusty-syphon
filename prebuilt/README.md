# Prebuilt binaries

Place prebuilt Syphon/Spout libraries here to avoid building from source. The build checks these directories **first** before running xcodebuild (macOS) or CMake (Windows).

## macOS — `prebuilt/macos/`

Put **Syphon.framework** (the whole framework bundle) in this folder:

```
prebuilt/macos/Syphon.framework/
```

So that the path `prebuilt/macos/Syphon.framework` exists. You can build it yourself from the `Syphon-Framework` Xcode project, or copy a prebuilt framework from another install.

**Override via env:** You can still set `SYPHON_FRAMEWORK_PATH` to a directory containing `Syphon.framework` (or to the framework path itself); that is checked before `prebuilt/macos/`.

## Windows — `prebuilt/windows/`

You can drop your Spout build output as-is. The build looks for **SpoutLibrary.lib** (for linking) and **SpoutLibrary.dll** (to copy next to your binary) and uses whatever layout you have.

**Supported layouts:**

1. **`lib/` and `bin/` folders** — drop the whole output (e.g. from CMake or your Spout build). The build will use `lib/SpoutLibrary.lib` and `bin/SpoutLibrary.dll`:
   ```
   prebuilt/windows/lib/SpoutLibrary.lib   (and any other .libs)
   prebuilt/windows/bin/SpoutLibrary.dll   (and any other .dlls)
   ```

2. **Flat** — put the two files directly in `prebuilt/windows/`:
   ```
   prebuilt/windows/SpoutLibrary.lib
   prebuilt/windows/SpoutLibrary.dll
   ```

3. **MD or MT subfolder** — if your Spout build has **MD** and **MT** folders, drop **one** of them (with its contents, including `lib/` and `bin/` if present):
   ```
   prebuilt/windows/MD/lib/SpoutLibrary.lib
   prebuilt/windows/MD/bin/SpoutLibrary.dll
   ```
   or the same under `prebuilt/windows/MT/`. The build checks `prebuilt/windows/`, then `prebuilt/windows/MD/`, then `prebuilt/windows/MT/` (first with a usable .lib wins).

**Which to use (MD vs MT)?**

- **MD** — Multi-threaded DLL runtime. Use this if you're building with the default Visual Studio / Rust MSVC toolchain (most common). Requires the VC++ redist on the machine.
- **MT** — Static runtime. Use this if you want to avoid depending on the VC++ redist; the C runtime is linked into the DLL. Must match how the rest of your app is built to avoid runtime conflicts.

For a typical `cargo build` on Windows with MSVC, **MD** is usually the right choice.
