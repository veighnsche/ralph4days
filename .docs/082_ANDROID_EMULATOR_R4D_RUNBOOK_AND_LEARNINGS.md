# Android Emulator + r4d Runbook and Learnings (2026-02-16)

## Scope
- Goal: fully install Android emulator tooling and run `r4d` on an Android emulator from this host.
- Host: Ultramarine Linux 43 (Wayland/KWin), kernel `6.18.8-cachyos1.fc43.x86_64`.
- Repo: `ralph4days`.

## Executive Summary
- Screen lock/screen-off was not the primary blocker.
- The emulator repeatedly crashed (`SIGSEGV`) in strict headless mode (`-no-window`) on this host.
- Stable startup required launching emulator with an explicit Wayland session context and hidden Qt window (`-qt-hide-window`), not strict no-window mode.
- `r4d` successfully installed and launched on emulator after clearing corrupted Kotlin incremental caches in Cargo registry plugin Android build dirs.

## What Failed
1. Repeated emulator crashes in strict headless mode:
- Binary: `qemu-system-x86_64-headless`
- Symptom: coredumps within ~9-15s during boot.
- Reproduced across:
  - API 35 and API 34 system images
  - GPU modes (`software`, `swiftshader`, `swangle`, `lavapipe`, `auto`, `off`)
  - `-accel off` only delayed crash.

2. `tauri android dev` with explicit device argument showed unstable behavior in this environment:
- `bunx tauri android dev emulator-5580 --no-watch` repeatedly stalled with little output.
- Running without explicit device argument worked once emulator/device detection was healthy.

3. Android build failure due corrupted Kotlin incremental caches:
- Error examples:
  - `Could not delete .../tauri-plugin-opener-2.5.3/android/build/.../caches-jvm`
  - `PersistentEnumerator storage corrupted .../tauri-plugin-dialog-2.6.0/android/build/.../lookups.tab`

## What Worked
1. Stable emulator launch (Wayland-aware, hidden window):

```bash
systemd-run --user --unit=r4d-emu-wayland --collect bash -lc '
  export XDG_RUNTIME_DIR=/run/user/1000
  export WAYLAND_DISPLAY=wayland-0
  export XDG_SESSION_TYPE=wayland
  exec $HOME/Android/Sdk/emulator/emulator \
    -avd r4d_api34 \
    -port 5580 \
    -qt-hide-window \
    -no-audio \
    -no-snapshot \
    -no-boot-anim \
    -gpu auto \
    -no-metrics
'
```

2. App deploy/run command:

```bash
export CI=1
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_SDK_ROOT=$HOME/Android/Sdk
export NDK_HOME=$HOME/Android/Sdk/ndk/27.0.12077973
export XDG_RUNTIME_DIR=/run/user/1000
export WAYLAND_DISPLAY=wayland-0
export XDG_SESSION_TYPE=wayland

bunx tauri android dev --no-watch
```

3. Successful install/launch evidence:
- `Performing Streamed Install`
- `Success`
- `Starting: Intent { cmp=com.vince.ralph/.MainActivity }`

4. Runtime verification:

```bash
adb -s emulator-5580 shell pm list packages | rg 'com.vince.ralph'
adb -s emulator-5580 shell dumpsys activity activities | rg 'ResumedActivity|mResumedActivity'
adb -s emulator-5580 shell pidof com.vince.ralph
```

5. Android Studio workflow (validated):
- Package install:

```bash
sudo dnf install -y android-studio
```

- Launcher command is `android-studio` (not `studio`).
- In this environment, launching from a non-graphical shell requires the graphical session env:

```bash
export XDG_RUNTIME_DIR=/run/user/1000
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus
export DISPLAY=:0
export WAYLAND_DISPLAY=wayland-0
export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=KDE
export XAUTHORITY=/run/user/1000/xauth_kOqVVL
```

- Open generated Android project directly:

```bash
android-studio /home/vince/Projects/ralph4days/src-tauri/gen/android
```

- Equivalent Tauri-driven Studio path:

```bash
bunx tauri android dev --open
```

- Verified signal in logs:
  - `External command line: /home/vince/Projects/ralph4days/src-tauri/gen/android`

## Full Runbook (Reproducible)
1. Ensure Android and Rust toolchains are installed:
- JDK 21, Android cmdline-tools, platform-tools, emulator, NDK 27, API 34+ system image, Rust Android targets.

2. Start emulator with Wayland context:
- Use `systemd-run` command shown above (`r4d-emu-wayland`).

3. Wait for device readiness:

```bash
adb devices -l
adb -s emulator-5580 shell getprop sys.boot_completed
```

4. If `sys.boot_completed=1`, run:
- `bunx tauri android dev --no-watch` with env exports from section above.

5. Verify app process + resumed activity (commands above).

## Recovery / Troubleshooting
1. If emulator crashes quickly with `-no-window`:
- Do not use strict headless on this host.
- Use Wayland-aware launch with `-qt-hide-window`.

2. If Gradle/Kotlin reports corrupted cache or cannot delete plugin build cache:

```bash
cd src-tauri/gen/android && ./gradlew --stop
rm -rf ~/.cargo/registry/src/index.crates.io-*/tauri-plugin-dialog-*/android/build
rm -rf ~/.cargo/registry/src/index.crates.io-*/tauri-plugin-opener-*/android/build
rm -rf ~/.cargo/registry/src/index.crates.io-*/tauri-plugin-fs-*/android/build
```

Then rerun:
- `bunx tauri android dev --no-watch`

3. If device appears `offline`:
- Wait a bit for boot completion.
- Recheck `adb devices -l`.
- Confirm emulator unit is still active:
  - `systemctl --user status r4d-emu-wayland.service --no-pager -n 20`

4. If `tauri android dev` hangs immediately:
- Retry without explicit device argument, letting Tauri auto-detect connected target.

## Audit Findings (Host)
- KVM available and loaded (`kvm_intel`, `/dev/kvm` present).
- Active Wayland session (`kwin_wayland`), but shell environment initially lacked display vars.
- Emulator 36.4.9 toolchain installed correctly.
- coredumps consistently tied to strict headless emulator startup path in this environment.

## Known Risk
- Root cause of `qemu-system-x86_64-headless` segfault in strict `-no-window` mode remains unresolved.
- Workaround is operationally reliable for now (`-qt-hide-window` + explicit Wayland env), but not a permanent root-cause fix.
