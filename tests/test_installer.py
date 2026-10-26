import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release_installer", Path(__file__).resolve().parents[1] / "install.py")
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)


class InstallerTests(unittest.TestCase):
    def fake_download(self, directory, corrupt=False, traversal=False, link=False, architecture="x86_64", minimum="2.36"):
        names = [f"kanata-studio-{kind}-0.2.0-linux-{architecture}.tar.gz" for kind in ("rust", "plugin")]
        for name in names:
            app = "studio-rust" in name
            root = f"kanata-studio-rust-0.2.0-linux-{architecture}" if app else "zoltan.kanata-studio"
            path = directory / name
            with tarfile.open(path, "w:gz") as archive:
                files = {root + "/install.py": "# test fixture\n"}
                if app:
                    files[root + "/manifest.json"] = json.dumps({"architecture": architecture, "minimumGlibc": minimum})
                if link:
                    entry = tarfile.TarInfo(root + "/escape")
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = "../../escaped"
                    archive.addfile(entry)
                if traversal:
                    files["../escaped"] = "must not escape"
                for filename, text in files.items():
                    data = text.encode()
                    entry = tarfile.TarInfo(filename)
                    entry.size = len(data)
                    archive.addfile(entry, io.BytesIO(data))
            digest = "0" * 64 if corrupt and not app else hashlib.sha256(path.read_bytes()).hexdigest()
            Path(str(path) + ".sha256").write_text(f"{digest}  {name}\n")

    def download(self, directory, **options):
        self.fake_download(directory, **options)
        metadata = {"tag_name": "v0.2.0", "assets": [{"name": p.name, "browser_download_url": "https://github.com/fixture/" + p.name} for p in directory.iterdir()]}
        with patch.object(installer, "release_metadata", return_value=metadata), patch.object(installer, "download_asset"), patch.object(installer.platform, "machine", return_value=options.get("architecture", "x86_64")), patch.object(installer.platform, "libc_ver", return_value=("glibc", "2.36")):
            return installer.download(directory, True)

    def test_both_archives_verified_and_extracted(self):
        with tempfile.TemporaryDirectory() as folder:
            app, plugin = self.download(Path(folder))
            self.assertTrue((app / "install.py").is_file())
            self.assertTrue((plugin / "install.py").is_file())

    def test_corrupt_plugin_prevents_even_app_extraction(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(RuntimeError, "Prüfsumme stimmt nicht"):
                self.download(Path(folder), corrupt=True)
            self.assertFalse((Path(folder) / "kanata-studio-rust-0.2.0-linux-x86_64").exists())

    def test_archive_cannot_escape_download_directory(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(RuntimeError, "Unsicherer Archiveintrag"):
                self.download(Path(folder), traversal=True)

    def test_archive_cannot_create_links(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(RuntimeError, "Unsicherer Archiveintrag"):
                self.download(Path(folder), link=True)

    def test_glibc_requirement_comes_from_manifest(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(RuntimeError, "glibc 2.44"):
                self.download(Path(folder), minimum="2.44")

    def test_aarch64_asset_selection(self):
        with tempfile.TemporaryDirectory() as folder:
            app, _ = self.download(Path(folder), architecture="aarch64")
            self.assertTrue(app.name.endswith("aarch64"))

    def test_missing_architecture_fails_before_download(self):
        with tempfile.TemporaryDirectory() as folder:
            with patch.object(installer, "release_metadata", return_value={"tag_name": "v0.2.0", "assets": []}), patch.object(installer, "download_asset") as fetch:
                with self.assertRaisesRegex(RuntimeError, "App-Paket"):
                    installer.download(Path(folder), False)
                fetch.assert_not_called()

    def test_foreign_download_url_is_rejected(self):
        with patch.object(installer, "open_url") as fetch:
            with self.assertRaisesRegex(RuntimeError, "Download-Adresse"):
                installer.download_asset("https://example.com/payload", Path("unused"))
            fetch.assert_not_called()

    def test_preflight_needs_no_gh_and_accepts_older_glibc(self):
        with patch.object(installer.platform, "system", return_value="Linux"), patch.object(installer.platform, "machine", return_value="x86_64"), patch.object(installer.platform, "libc_ver", return_value=("glibc", "2.35")), patch.object(installer.shutil, "which", return_value=None):
            installer.preflight()

    def main_calls(self, arguments):
        with patch.object(sys, "argv", ["install.py"] + arguments), patch.object(installer, "preflight"), patch.object(installer.shutil, "which", return_value="/fake/tool"), patch.object(installer, "download", return_value=(Path("/fixture/app"), None if "--app-only" in arguments else Path("/fixture/plugin"))), patch.object(installer, "run") as run:
            installer.main()
            return [call.args[0] for call in run.call_args_list]

    def test_default_installs_both_and_restarts_shell(self):
        self.assertEqual(self.main_calls([]), [
            ["omarchy", "plugin", "validate", "/fixture/plugin"],
            [sys.executable, "/fixture/app/install.py"],
            [sys.executable, "/fixture/plugin/install.py", "--enable"],
            ["omarchy", "restart", "shell"],
        ])

    def test_check_only_does_not_install_or_restart(self):
        self.assertEqual(self.main_calls(["--check-only"]), [["omarchy", "plugin", "validate", "/fixture/plugin"]])

    def test_app_only_and_deferred_restart(self):
        self.assertEqual(self.main_calls(["--app-only"]), [[sys.executable, "/fixture/app/install.py"]])
        self.assertNotIn(["omarchy", "restart", "shell"], self.main_calls(["--no-restart"]))

    def test_custom_prefix_reaches_bundle_installer(self):
        self.assertEqual(self.main_calls(["--app-only", "--prefix", "/tmp/custom prefix"]), [[sys.executable, "/fixture/app/install.py", "--prefix", "/tmp/custom prefix"]])

    def test_other_distros_install_without_omarchy(self):
        with patch.object(sys, "argv", ["install.py"]), patch.object(installer, "preflight"), patch.object(installer.shutil, "which", return_value=None), patch.object(installer, "download", return_value=(Path("/fixture/app"), None)) as download, patch.object(installer, "run") as run:
            installer.main()
            self.assertFalse(download.call_args.args[1])
            run.assert_called_once_with([sys.executable, "/fixture/app/install.py"])


if __name__ == "__main__":
    unittest.main()
