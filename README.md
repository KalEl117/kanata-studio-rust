# Kanata Studio Rust

[Deutsch](README.de.md)

A graphical editor for existing Kanata configurations, with a Rust core, a
Qt Quick desktop app, and an optional Omarchy plugin. The bar popup provides a
compact keyboard editor with tap/hold actions, Kanata expressions, undo, and
saving. The full editor also includes macros and a 30-second test mode.

## Install on Linux

The standalone app runs on Wayland or X11 and does not require Omarchy or
Hyprland. Download packages include Qt and QML. Installation is per user and
does not require `sudo`.

### Quick installation

Requirements: Linux with glibc, Python 3.10+, and a published package for your
CPU architecture. You do not need a GitHub account, GitHub CLI, Rust, or a compiler.

```sh
curl -fL https://raw.githubusercontent.com/KalEl117/kanata-studio-rust/main/install.py -o install.py
python3 install.py
```

You can also download `install.py` from this repository in your browser and run
it with Python. The installer downloads the latest stable GitHub release,
verifies all SHA-256 checksums, extracts the packages, and adds an application
launcher. Run it again to update an existing installation.

On other Linux desktops, it installs the app only. On Omarchy, it also installs
and enables the matching plugin and restarts the shell.

**Since v0.2.1:** The x86_64 package requires **glibc 2.36+**. The previous
v0.2.0 release required glibc 2.44. The installer can select aarch64 assets when
available, but the current build workflow produces x86_64 packages only.

Options:

```sh
python3 install.py --check-only          # Download and verify without installing
python3 install.py --app-only            # Skip the plugin, including on Omarchy
python3 install.py --release v0.2.1       # Select a specific release
python3 install.py --prefix ~/.local     # Choose the app installation prefix
python3 install.py --no-restart          # Restart the Omarchy shell manually later
```

Each package declares its minimum glibc version in `manifest.json`. Incompatible
packages are rejected before installation, showing the required and installed
versions. Alpine/musl systems need a separate source build.

Default installation paths:

- App: `~/.local/share/kanata-studio-rust`
- Desktop entry: `~/.local/share/applications/kanata-studio-rust.desktop`
- Command: `~/.local/bin/kanata-studio-rust`

If `~/.local/bin` is not in your `PATH`, use the application launcher or the full
command path.

### Set up Kanata separately

The app and plugin do not install Kanata, configure keyboard permissions, set up
a keyboard service, or transfer your personal keymap. To validate, save, and
live-test configurations, you need Kanata and a configured **systemd user service
named `kanata.service`** that uses the configuration file you open in the editor.
The current service integration does not support a system-wide Kanata service
on its own. An automatic Kanata and permissions setup wizard is not included.

The Omarchy plugin opens `~/.config/kanata/kanata.kbd` by default, falling back to
`~/.config/kanata/config.kbd`. See the [plugin guide](omarchy-plugin/README.md).

### Install an archive manually

Download the app archive and its matching `.sha256` file from
[Releases](https://github.com/KalEl117/kanata-studio-rust/releases).
For example, for v0.2.1, run these commands in your download directory:

```sh
sha256sum -c kanata-studio-rust-0.2.1-linux-x86_64.tar.gz.sha256
tar -xzf kanata-studio-rust-0.2.1-linux-x86_64.tar.gz
python3 kanata-studio-rust-0.2.1-linux-x86_64/install.py
~/.local/bin/kanata-studio-rust
```

Alternatively, run `launch.sh` directly from the extracted directory. Python is
only needed for installation. The bundle includes Qt, QML modules, platform
plugins, and required runtime libraries. Your system provides graphics drivers
and a Wayland or X11 session.

## Build packages for other distributions

The [Portable Linux bundle workflow](.github/workflows/linux-bundle.yml) builds
on Debian 12 with Qt 6.4 and checks that the measured minimum glibc version is no
higher than 2.36. This baseline targets Debian 12+, Ubuntu 24.04+ and corresponding
Linux Mint versions, Fedora, and Arch. Ubuntu 22.04, with glibc 2.35, is outside
this build baseline.

The workflow runs on pull requests, `main`, version tags, and manual dispatch. It
checks the extracted bundle, installation in a temporary directory, and preview
recovery. Additional container tests on Debian 12, Ubuntu 24.04, and Fedora run
the finished package without system Qt. These checks do not exercise a real
Wayland/X11 desktop or replace graphics-driver testing on target machines.

After a successful run, the `linux-x86_64-glibc-2.36` Actions artifact contains
the app and plugin archives, checksums, and download installer. The workflow does
not publish releases automatically. Upload these files as release assets using
the matching version from `Cargo.toml`.

On Omarchy, the download installer also needs the plugin archive and checksum
for the same release; otherwise, use `--app-only`. CI packages the plugin with
`--skip-validation` because Omarchy is not installed there. The target machine
still validates the plugin with Omarchy before installation. Local builds on
Omarchy remain supported, but inherit the build machine's glibc requirement.

Qt locates its modules and plugins through `qt.conf`; see the
[Qt documentation](https://doc.qt.io/qt-6/qt-conf.html). Libraries are loaded
dynamically, and available distribution license notices are included under
`licenses/`. Native tools such as Kanata and systemctl retain their original
library environment.

Run `~/.local/bin/kanata-studio-rust --display-check` to inspect the windowing
protocol and per-screen scaling. Qt 6 handles scaling automatically; setting
`QT_SCALE_FACTOR` multiplies it again. See [Qt High DPI](https://doc.qt.io/qt-6/highdpi.html).

### Build from source

Requirements: Git, Python 3.11+, Rust 1.88+, Cargo, a C++ compiler, make, qmake6,
and Qt 6.4+ with Quick, Quick Controls 2, and Concurrent.

On Debian 12 / Ubuntu 24.04:

```sh
sudo apt install git python3 build-essential qt6-base-dev qt6-declarative-dev qt6-wayland \
  qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-templates \
  qml6-module-qtquick-window qml6-module-qtquick-layouts qml6-module-qtquick-dialogs \
  qml6-module-qtquick-shapes qml6-module-qtqml qml6-module-qtqml-models \
  qml6-module-qtqml-workerscript qml6-module-qt-labs-settings qml6-module-qt-labs-platform
```

On Arch/Omarchy:

```sh
sudo pacman -S --needed git python base-devel rust qt6-base qt6-declarative qt6-wayland
```

Make sure Rust 1.88+ is available, for example through an existing rustup
installation. Then clone and build:

```sh
git clone https://github.com/KalEl117/kanata-studio-rust.git
cd kanata-studio-rust
cargo fetch --locked
./build.sh
./run.sh
./package.sh --install
```

To install the Omarchy plugin as well:

```sh
python3 tools/prepare-omarchy-plugin.py
python3 tools/install-plugin.py --source dist/omarchy-plugin/zoltan.kanata-studio --enable
omarchy restart shell
```

## Development

Install the build dependencies above and run `cargo fetch --locked` on a new
machine before starting development.

```sh
cd kanata-studio-rust
./run.sh
```

You can pass a `.kbd` file as an argument. Without one, the app opens
`~/.config/kanata/kanata.kbd` or `config.kbd`; it does not create a configuration.
`run.sh` launches the existing executable and builds only on first launch or
after changes to source, build files, or QML. `./build.sh` always runs the build
checks. Builds use dependencies from `Cargo.lock` offline.

```sh
./package.sh           # Create a runtime bundle and checksum
./package.sh --install # Also install it for the current user
```

The packaging tool requires Linux, Python 3.11+, `ldd`, `readelf`, and an existing
Qt installation. Available distribution license notices are collected from the
package database on Arch/Omarchy and Debian.

The UI lives in `ui/main.qml` and `ui/ActionPanel.qml`. During the build, small
adaptations bind key labels to the prepared Rust model without modifying these
source QML files.

## Editing and data model

Clicking a key selects it without creating a draft or a history entry. A missing
key is added to the shared key list and all layers only when you assign an
output, tap/hold action, or macro. Adding it and making its first assignment is
one undo/redo step, and selection is preserved. Other layers start with
passthrough (`_`). Only testing or saving changes the active configuration.

The keyboard and editing panel distinguish normal behavior, custom assignments,
and assignments inherited from the base layer. Unconfigured keys remain readable
and selectable. For passthrough, the editor previews the first base layer and
shows its origin; runtime output follows the currently active base layer.

Keys that differ from the saved configuration have a gold outline and a dot.
Hovering shows the saved and draft outputs, including tap/hold actions. New keys
and layers are marked. Changes to an inherited base assignment also appear on
passthrough layers. Undo and saving update these markers; comments and formatting
changes alone do not mark keys as changed.

Tap/hold assignments show separate tap and hold descriptions on each key, such
as tapping Esc and holding a layer switch. Temporarily held layers are identified
as such. Embedded actions and base previews use the same labels, and tooltips
show longer descriptions in full.

The output picker uses valid, case-sensitive Kanata key names, including symbols
and Shift/AltGr combinations. On a German layout, `Slash` means the physical minus
key and `Minus` means the ß key.

Rust manages the selected key and layer. The parsed document stays in memory
when selection changes. The UI reads prepared actions, labels, and action details;
repeated field access does not reparse the document. Document, selection, actions,
colors, status, and countdown have separate change signals, so status and countdown
updates do not rebuild the keyboard view.

Periodic layout, device, and theme detection runs outside the GUI thread. Initial
detection runs before the window opens. German, US English, UK English, ThinkPad,
and automatic detection are available. Layout selection is stored in
`~/.config/kanata-studio-rust/settings.json`; an existing selection from the first
Qt/Rust prototype is migrated. UI language and panel settings remain in Qt.

Comments and untouched file sections are preserved. Renaming a layer updates
direct references. Layers with remaining direct references cannot be deleted.
Indirect references through variables or aliases still require final validation
with Kanata.

## Saving and test mode

Validation uses a temporary file in the configuration directory and limits the
Kanata process runtime. Saving writes atomically with the original permissions.
External changes are detected; if reloading the service fails, the original file
is restored.

The 30-second test runs in a separate process. Rust handles expiry, cancellation,
and recovery after interruptions. `kanata-studio-rust-recover.service` uses
`~/.local/state/kanata-studio-rust/preview`, runs before `kanata.service`, and starts
the launcher with `--recover-preview`. Service setup is repeated only when needed
after changes or incomplete setup. Every normal startup checks for interrupted
tests.

After startup recovery, the editor reloads the restored file. Externally changed
files are not overwritten, and recovery data is retained for investigation.
Avoid editing the same file with another Studio version while saving or testing.

## Checks

```sh
~/.cargo/bin/cargo test --locked --offline
~/.cargo/bin/cargo clippy --locked --offline --all-targets -- -D warnings
~/.cargo/bin/cargo fmt --check
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --self-test
python3 tools/check-recovery.py
python3 tools/check-package.py
```

Rust tests cover parsing, editing, selection, detection, settings, atomic saving,
service failures, and recovery. The Qt self-test uses
`tests/fixtures/wrapped-layout.kbd` by default and temporary copies with a simulated
service. `check-recovery.py` activates a temporary configuration, kills the real
test process with SIGKILL, and checks recovery on the next startup, editor contents,
and protection of external changes. These tests do not modify the active keyboard
configuration or the real Kanata service.

You can run the recovery check against an extracted or installed bundle:
`python3 tools/check-recovery.py /path/to/launch.sh`. Package checks also exercise
the installer and launcher after installation in a temporary directory.

## Measurements

```sh
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --benchmark-ui tests/fixtures/wrapped-layout.kbd
KANATA_STUDIO_STARTUP_PROFILE=1 ./run.sh
```

The interaction benchmark also checks the QML flow for selecting a missing key,
assigning its first output, and undoing/redoing the change completely. It measures
60 key selections, layer switches, and macro changes with undo, including UI
updates. It discards drafts without writing configuration files. Measurements
cover call processing and immediate Qt/QML updates, not complete input-to-display
latency. Results for the test and active configurations are in
`reports/interaction.json`.

Startup profiling reports time to Qt initialization, configuration loading,
recovery readiness, UI loading, and the first frame. Timing begins inside the
program and excludes any required compilation.

Create a preview image without setting up recovery services:

```sh
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --screenshot /tmp/kanata-studio-rust.png
```
