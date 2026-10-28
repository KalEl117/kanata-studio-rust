# Kanata Studio Rust

Original project code: [MIT](LICENSE). The desktop app uses Qt under LGPLv3;
dependencies retain their own licenses. See [third-party notices](THIRD_PARTY_NOTICES.md)
and [binary redistribution instructions](LICENSING.md).

[Deutsch](README.de.md)

A graphical editor for existing Kanata configurations, with a Rust core, a
Qt Quick desktop app, and an optional Omarchy plugin. The bar popup provides a
compact keyboard editor with tap/hold actions, Kanata expressions, undo, and
saving. The full editor also includes macros and a 30-second test mode.

## Install on Linux

The standalone app runs on Wayland or X11 and does not require Omarchy or
Hyprland.

Download packages include Qt and QML. Installation is per user and does not
require `sudo`.

### Quick installation

Requirements: x86_64 Linux with glibc 2.36+ and Python 3.10+. You do not need a
GitHub account, GitHub CLI, Rust, or a compiler.

```sh
curl -fL https://raw.githubusercontent.com/KalEl117/kanata-studio-rust/main/install.py -o install.py
python3 install.py
```

The installer downloads the latest stable GitHub release, verifies all SHA-256
checksums, extracts the packages, and adds an application launcher. Run it again
to update an existing installation. On other Linux desktops, it installs the app
only. On Omarchy, it also installs and enables the matching plugin and restarts
the shell.

```sh
python3 install.py --check-only        # Download and verify without installing
python3 install.py --app-only          # Skip the plugin, including on Omarchy
python3 install.py --release v0.2.2    # Select a specific release
python3 install.py --prefix ~/.local   # Choose the app installation prefix
python3 install.py --no-restart        # Restart the shell yourself later
```

v0.2.2 is the first release with complete license and corresponding-source
materials. The v0.2.0 and v0.2.1 archives were withdrawn on 2026-10-28; do not use
or redistribute copies of them. See [LICENSING.md](LICENSING.md).

### Set up Kanata separately

The app and plugin do not install Kanata, configure keyboard permissions, set up
a keyboard service, or transfer your personal keymap. To validate, save, and
live-test configurations, you need Kanata and a configured **systemd user service
named `kanata.service`** that uses the configuration file you open in the editor.
The current service integration does not support a system-wide Kanata service
on its own. An automatic Kanata and permissions setup wizard is not included.

The Omarchy plugin opens `~/.config/kanata/kanata.kbd` by default, falling back to
`~/.config/kanata/config.kbd`. See the [plugin guide](omarchy-plugin/README.md).

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
a new version from `Cargo.toml`; do not reuse the withdrawn v0.2.0/v0.2.1 tags.

On Omarchy, the download installer also needs the plugin archive and checksum
for the same release; otherwise, use `--app-only`. CI packages the plugin with
`--skip-validation` because Omarchy is not installed there. The target machine
still validates the plugin with Omarchy before installation. Local builds on
Omarchy remain supported; private bundles require `--development` and inherit
the build machine's glibc requirement.

Qt locates its modules and plugins through `qt.conf`; see the
[Qt documentation](https://doc.qt.io/qt-6/qt-conf.html). Libraries are loaded
dynamically. New release builds include complete distribution copyright files
under `licenses/` and matching source packages under `sources/debian/`.
The withdrawn v0.2.0/v0.2.1 archives predate this mechanism; see
[LICENSING.md](LICENSING.md). Native tools such as Kanata and systemctl retain
their original library environment.

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
./package.sh --development --install # Private local installation
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
./package.sh                         # Debian with matching deb-src: release bundle + sources
./package.sh --development --install # Private local bundle (not for redistribution)
```

The packaging tool requires Linux, Python 3.11+, `ldd`, `readelf`, and an existing
Qt installation. Release bundles require Debian with matching `deb-src`
repositories and network access for the exact library source packages. Sources
are included in the archive and increase its download and installed size.
See [LICENSING.md](LICENSING.md) for the source and replacement instructions.

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
detection runs before the window opens. Automatic display follows the device
filters of the open Kanata configuration: among the edited keyboards, USB is
preferred over other external keyboards, and those over the built-in keyboard.
Plugging and unplugging updates the main window and an open Omarchy popup without
losing drafts or the selected layer. Excluded devices stay visible with their
status. If only excluded keyboards are connected, no made-up editable keyboard is
shown.

Both UIs share one layout model (`ui/KeyboardModel.js`). ThinkPad, full-size, and
TKL are selectable; German, US English, and UK English remain available as manual
display options. ANSI Enter sits in the middle letter row, ISO Enter spans two
rows, and navigation and numpad also appear in the popup. Choosing a language no
longer turns an external keyboard into a laptop keyboard. Hyprland's normalized
device names and the active layout index are taken into account. The printed
language, freely programmed firmware layouts, and the exact form factor of unknown
keyboards cannot be derived reliably from HID key codes; use the manual selection
if the form factor differs. Known firmware names such as Adv360 Pro are not
classified as full-size keyboards because of possible numpad output.

The layout selection in `~/.config/kanata-studio-rust/settings.json` only changes
the display, not the device filters. Other settings in this file are preserved.
UI language and panel settings remain in Qt.

### Capture ordinary keyboards automatically, exclude firmware keyboards

Kanata itself handles plugging and unplugging, even while Studio is closed. For an
Advantage360 Pro, for example, the fixed allow list can be replaced with these
options in the existing `defcfg` (keep the other options):

```lisp
linux-device-detect-mode keyboard-only
linux-continue-if-no-devs-found yes
linux-dev-names-exclude ("Adv360 Pro Keyboard" "Adv360 Pro")
```

Remove `linux-dev` and `linux-dev-names-include` for this automatic selection.
Kanata compares full device names; add other firmware keyboards with their actual
names. No general HID property identifies every programmable keyboard. Studio's
detection of known firmware names is a display hint and never changes the active
configuration on its own. The options are described in the
[Kanata configuration guide](https://github.com/jtroo/kanata/blob/v1.12.0/docs/config.adoc#linux-only-linux-dev-names-exclude).
Restart Kanata after changing device filters; a reload is not enough.
`keyboard-only` skips combined mouse/keyboard interfaces; if a real keyboard only
offers such an interface, select the device explicitly.

Comments and untouched file sections are preserved. Renaming a layer updates
direct references. Layers with remaining direct references cannot be deleted.
Indirect references through variables or aliases still require final validation
with Kanata.

Layers defined with `deflayermap` are shown in file order alongside `deflayer`
layers. Explicit mappings take precedence over `_`/`___`; unmapped keys are
transparent, as in Kanata. In the key editor, changing a key with its own mapping
replaces only that action. Otherwise a `key action` entry is added to the map.
The `_`/`___` fallback is never changed because it also applies to other keys.
Map layers can be renamed, deleted, and copied as a new map layer. New physical
keys only add an entry to a map layer when they get an action there.

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
node --test tests/keyboard-model.test.cjs
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --self-test
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= qt/kanata-studio-rust --ui-check tests/fixtures/wrapped-layout.kbd
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
