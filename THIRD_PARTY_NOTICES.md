# Third-party notices

Kanata Studio Rust's original code, documentation, and Omarchy plugin are
copyright (c) 2026 Zoltan Jo (KalEl117), licensed under [MIT](LICENSE).
This license does not replace the licenses of third-party components.

## Rust dependencies

[licenses/RUST-DEPENDENCIES.txt](licenses/RUST-DEPENDENCIES.txt) contains the exact
versions, upstream references, copyright notices, and license texts of every
crate in Cargo.lock, including transitive and build dependencies. Most offer
MIT or Apache-2.0; unicode-ident additionally requires Unicode-3.0, and zmij is
MIT. These notices are included in both the application and plugin packages.
Binary packages also contain the actual Rust toolchain's copyright information
and license texts under `licenses/rust-toolchain/`, covering its standard library
and included third-party code.

## Qt and native runtime libraries

The desktop application uses **Qt under the GNU Lesser General Public License
version 3 (LGPLv3)**. It dynamically links Qt Core, Gui, Network, Qml, Quick,
Quick Controls 2, Concurrent, and their runtime dependencies. The portable
package also includes QML modules and platform, image, and other Qt plugins.
Qt copyright belongs to The Qt Company and other contributors; individual
third-party components retain their own notices and licenses.

The LGPLv3 and its incorporated GPLv3 text are included in
`licenses/LGPL-3.0.txt` and `licenses/GPL-3.0.txt`. The package-specific
`licenses/NATIVE-PACKAGES.json` maps **all copied native files**, including QML
and plugins, to their distribution packages, exact versions, and copyright
files under `licenses/native/`. Full Debian common license texts are included
under `licenses/debian-common/`; references to `/usr/share/common-licenses/` in
Debian copyright files refer to those same texts.

Redistributable Debian bundles contain the corresponding native library source
packages, including distribution patches and build recipes, in `sources/debian/`.
See [LICENSING.md](LICENSING.md) for extraction, rebuilding, and replacement.
You may modify and replace the LGPL libraries and reverse engineer the combined
work to debug those modifications. This project imposes no additional restriction
on those rights. There is no signature check preventing replacement of libraries.

Development bundles explicitly marked `redistributable: false` are for private
local testing and do not contain the complete native redistribution materials.
Do not publish them as releases.

## Separately installed software

Kanata is a separate program invoked as a subprocess; neither its executable nor
its source is included in application or plugin release packages. Its upstream
license is LGPLv3: <https://github.com/jtroo/kanata>. The integration and naming
do not imply endorsement by the Kanata project.

Omarchy, omarchy-shell/Quickshell, Hyprland, systemd, the host's fonts, graphics
drivers, and glibc are supplied separately by the user's system and retain their
own licenses. Referencing a font family does not bundle that font. The plugin
contains this project's QML and Rust helper; it does not redistribute the shell
or Qt runtime. If any external code, fonts, icons, or binaries are added later,
their provenance and redistribution requirements must be reviewed separately.
