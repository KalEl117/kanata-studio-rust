# Kanata Studio Omarchy plugin

Original plugin code: MIT (see LICENSE in the repository or prepared package).
Prepared packages also include THIRD_PARTY_NOTICES.md and licenses/ with Rust
notices. Omarchy/Quickshell and Qt are supplied separately by the host system.

[Deutsch](README.de.md) · [Main installation guide](../README.md)

The Omarchy bar widget shows the status of `kanata.service`. Left-click to open
or close a compact popup with Kanata/backend status, refresh, and an option to
open the installed Rust app. Clicking outside closes it. Position, colors, and
sizing follow the Omarchy bar and theme.

The keyboard uses the Studio app's geometry, including ISO Enter, function-key
groups, and ThinkPad arrow keys. Click a key to edit simple outputs, tap/hold
actions, or Kanata expressions. Applying an edit updates the draft without
changing the active configuration. Gold markers highlight changes. Undo, redo,
and discard are available in the popup.

Saving validates the draft with Kanata and checks that `kanata.service` uses the
file. It writes atomically, reloads the service, and restores the previous file
if reloading fails. External file changes and active Studio keyboard tests block
saving. Selecting a layer in the editor does not switch the active keyboard
layer. Passthrough previews use the first base layer; runtime behavior follows
the currently active base. Source keys missing from the layout appear below it.

Applied drafts are saved under `~/.local/state/kanata-studio-plugin/drafts` with
0600 permissions and restored when reopening after a shell restart. Undo history
belongs to the current backend session; drafts survive restarts, undo history
does not. Unapplied field text is retained when closing the popup within the
same shell session. Apply or reset it before switching keys or layers. Open the
full Studio editor for macro dialogs and the 30-second test mode.

`Backend.qml` runs the Rust helper as a separate process, checks protocol
version 1, and serializes requests. Errors and timeouts are displayed.

## Install

The [main download installer](../README.md#quick-installation) installs the
matching plugin automatically on Omarchy. Kanata and a configured
`kanata.service` user service are required for validation and saving.

## Build and check

From the repository root:

```sh
python3 tools/prepare-omarchy-plugin.py
python3 tools/check-omarchy-plugin.py
```

The first script builds only the Rust helper offline and prepares the installable
directory at `dist/omarchy-plugin/zoltan.kanata-studio`. It requires Cargo, cached
dependencies, and Omarchy for validation. The finished plugin does not require
Rust, Cargo, Node, or a separate Qt runtime: it uses the existing
Omarchy/Quickshell runtime. The binary must match the target architecture and
glibc version. Status queries require systemd; opening the full editor requires
the separately installed Rust app. Building does not install or activate the
plugin in the running shell.

To install the prepared plugin:

```sh
python3 tools/install-plugin.py --source dist/omarchy-plugin/zoltan.kanata-studio --enable
omarchy restart shell
```

Alternatively, **copy** the prepared directory to
`~/.config/omarchy/plugins/zoltan.kanata-studio` without symlinks, then run:

```sh
omarchy-shell shell rescanPlugins
omarchy plugin enable zoltan.kanata-studio --section right
```

To use a custom editor path, set `studioExecutable` in the corresponding
`shell.json` bar entry:

```json
{"id":"zoltan.kanata-studio", "studioExecutable":"/absolute/path/to/kanata-studio-rust"}
```

To open a different saved configuration, set `configPath` to an absolute `.kbd`
path in the same entry. Without it, the overview uses
`~/.config/kanata/kanata.kbd`, falling back to `config.kbd`.

## Use the backend

Without arguments, `kanata-studio-helper` handles one request. With `--serve`, it
reads one JSON request per line and immediately writes one JSON response per
line, keeping stdin open. The request `id` is echoed unchanged. Blank lines are
ignored; invalid requests return `ok:false`.

All existing core operations remain available. Direct calls must supply `source`;
the QML integration adds it from the last response. `overview` reads the file at
an optional `path` and returns all layer views, normalized key names, and layout
data independently of editor history.

```json
{"id":1,"op":"ping"}
{"id":2,"op":"loadView","source":"(defsrc a)\n(deflayer base a)\n"}
{"id":3,"op":"setAction","source":"(defsrc a)\n(deflayer base a)\n","expression":"esc"}
```

In QML, once `ready === true`:

```js
backend.loadSource(text) // Clear history and load the document
backend.request({op: "select", layer: 0, key: 0})
backend.request({op: "setAction", expression: "esc"})
backend.request({op: "undo"})
```

Responses arrive through `replied(requestId, response)`; errors use
`failed(requestId, message)`. `source`, `view`, `canUndo`, and `canRedo` reflect
successful responses. Load a valid document before using the editor. Long-running
preview workers belong in a **separate process**, outside the serial editor
connection.

The `plugin*` operations maintain their own editor state: `pluginOpen`,
`pluginSelect`, `pluginLayer`, `pluginSetAction`, `pluginSetTapHold`, `pluginUndo`,
`pluginRedo`, `pluginDiscard`, and `pluginSave`. Responses include a `document`
with layer views, selection, actions, and draft status. Saving a draft writes
only its state file; only `pluginSave` changes the Kanata file and reloads the
service.

## Planned improvements

- Bring the visual output picker and macro dialog from the Studio app into the popup.
- Integrate the 30-second test with a separate preview worker and recovery.
- Check additional layouts and larger multi-monitor sessions.
