# Licensing and binary redistribution

The original project code is MIT-licensed; third-party code remains under its
own licenses. See [LICENSE](LICENSE) and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). This document describes the
packaging process; it is not a legal warranty or an assertion that MIT covers
every file in a binary bundle.

## Reproducible notices

After changing Cargo.lock, run `cargo fetch --locked`, then
`python3 tools/license_materials.py --write`. Review the resulting changes.
`python3 tools/license_materials.py` verifies the checked-in notices against
the actual dependency manifests and original license files. Unknown Rust license
expressions stop the check for review. Both application and plugin packaging run
this check and copy the notices, project license, and Rust runtime notices.

## Portable releases

Build redistributable bundles using the Debian 12 workflow in
`.github/workflows/linux-bundle.yml`. Its `deb-src` repositories must match the
binary repositories. `./package.sh` records the owners of every copied native
file and fetches the **exact source package version** with `apt-get source`.
The bundle contains each source descriptor, upstream tarball, Debian patches
and packaging, as applicable, under `sources/debian/`. Download failure, missing
ownership/copyright information, or an unreviewed Qt module stops packaging.
The manifest records `redistributable: true` only after this collection succeeds;
that flag describes packaging completeness, not a legal certification.

Source files are included inside the application archive, so every recipient
receives them with the binaries. Do not strip `sources/`, `licenses/`, LICENSE,
THIRD_PARTY_NOTICES.md, or LICENSING.md when repackaging. Keep the published
archive and its checksum together. No generic upstream link or promise to supply
sources later is used as a substitute for the exact source packages.

The source inventory and SHA-256 hashes are in `licenses/NATIVE-PACKAGES.json`.
For a source package, enter its directory and run, for example:

```sh
dpkg-source -x qt6-base_*.dsc
```

Install the dependencies listed in the extracted `debian/control` (on a matching
Debian build system, `apt-get build-dep qt6-base` installs the current repository
build dependencies). Inside the extracted directory, `dpkg-buildpackage -b -uc -us`
uses its Debian build rules. The same approach applies to the other source
packages. Consult their README/build documentation for upstream builds or older
versions whose dependencies are no longer in the current repositories.

To use a modified LGPL library, close the app, extract a writable copy of the
application archive, replace the matching files in `lib/`, `plugins/`, and/or
`qml/` with ABI-compatible rebuilt versions, then run `./launch.sh`. `qt.conf`
and the launcher select those directories. No signing key or authorization from
this project is required. If you change the Qt ABI, rebuild the application
using the source and instructions in README.md, against your Qt installation.
The release's `manifest.json` records the project commit and Qt version.

On other build hosts, `./package.sh --development` creates a private test bundle
marked `redistributable: false`; it must not be published. Running `./build.sh`
and `./run.sh` against system Qt does not create a redistributable runtime bundle.

The CI validator is a separately built Kanata executable used only during build
tests. It is not uploaded in the release artifact. Cross-distribution smoke
tests exercise display detection and QML rendering; full validation and recovery
tests run in the Debian build job with the upstream validator.

## Existing releases

The app/plugin archives, checksums, and release installers for v0.2.0 and v0.2.1
were removed from GitHub Releases on 2026-10-28 because they predated this
collection and lacked complete redistribution materials. Their version tags
and source history are retained. v0.2.2 is the
first replacement release built with the verified process above.

Adding license files to this repository does not retroactively repair copies
of the old archives. Do not re-upload them. A replacement binary release must
use a new version and the verified packaging process above.

## References

- Qt licensing: <https://doc.qt.io/qt-6/licensing.html>
- Qt LGPL obligations: <https://www.qt.io/development/open-source-lgpl-obligations>
- LGPLv3: <https://www.gnu.org/licenses/lgpl-3.0.html>
- GPLv3, including source distribution options: <https://www.gnu.org/licenses/gpl-3.0.html>
