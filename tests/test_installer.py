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
    def fake_download(self, command, corrupt=False, traversal=False):
        directory = Path(command[command.index("--dir") + 1])
        names = [command[i + 1] for i, item in enumerate(command) if item == "--pattern" and command[i + 1].endswith(".tar.gz")]
        for name in names:
            app = "studio-rust" in name
            root = "kanata-studio-rust-0.2.0-linux-x86_64" if app else "zoltan.kanata-studio"
            path = directory / name
            with tarfile.open(path, "w:gz") as archive:
                files = {root + "/install.py": "# test fixture\n"}
                if app:
                    files[root + "/manifest.json"] = json.dumps({"architecture": "x86_64", "minimumGlibc": "2.44"})
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
        with patch.object(installer, "run", side_effect=lambda args: self.fake_download(args, **options)), patch.object(installer.platform, "machine", return_value="x86_64"), patch.object(installer.platform, "libc_ver", return_value=("glibc", "2.44")):
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
            with self.assertRaises(tarfile.FilterError):
                self.download(Path(folder), traversal=True)

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


if __name__ == "__main__":
    unittest.main()
