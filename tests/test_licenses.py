"""Regression tests for missing license material and exact file ownership."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('license_materials', ROOT / 'tools/license_materials.py')
licenses = importlib.util.module_from_spec(spec)
spec.loader.exec_module(licenses)


class LicensingTests(unittest.TestCase):
    def test_lock_digest_matches_checked_in_notices(self):
        import hashlib
        digest = hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest()
        self.assertIn('Cargo.lock SHA-256: ' + digest,
                      (ROOT / 'licenses/RUST-DEPENDENCIES.txt').read_text())

    def test_missing_upstream_license_stops_generation(self):
        with tempfile.TemporaryDirectory() as directory:
            metadata = {'workspace_members': [], 'packages': [{
                'id': 'test', 'name': 'test', 'version': '1', 'license': 'MIT',
                'manifest_path': directory + '/Cargo.toml'}]}
            with patch.object(licenses, 'command', return_value=json.dumps(metadata)):
                with self.assertRaisesRegex(RuntimeError, 'Review license manually'):
                    licenses.rust_notices()

    def test_unknown_license_requires_review(self):
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, 'LICENSE').write_text('Some license')
            metadata = {'workspace_members': [], 'packages': [{
                'id': 'test', 'name': 'test', 'version': '1', 'license': 'LicenseRef-Unknown',
                'manifest_path': directory + '/Cargo.toml'}]}
            with patch.object(licenses, 'command', return_value=json.dumps(metadata)):
                with self.assertRaisesRegex(RuntimeError, 'Unreviewed Rust license'):
                    licenses.rust_notices()

    def test_merged_usr_and_architecture_are_preserved(self):
        paths = {Path('/usr/lib/example.so'), Path('/usr/lib/qt6/qml/Example/qmldir')}
        result = subprocess.CompletedProcess([], 1,
            'example:amd64: /lib/example.so\nqml6-example:amd64: /usr/lib/qt6/qml/Example/qmldir\n', '')
        with patch.object(licenses.subprocess, 'run', return_value=result):
            owners = licenses.debian_owners(paths)
        self.assertEqual(owners[Path('/usr/lib/example.so')], 'example:amd64')
        self.assertEqual(owners[Path('/usr/lib/qt6/qml/Example/qmldir')], 'qml6-example:amd64')

    def test_missing_and_ambiguous_owners_stop_packaging(self):
        for output in ('', 'one: /usr/lib/example.so\ntwo: /usr/lib/example.so\n',
                       'one, two: /usr/lib/example.so\n'):
            with self.subTest(output=output):
                result = subprocess.CompletedProcess([], 0, output, '')
                with patch.object(licenses.subprocess, 'run', return_value=result):
                    with self.assertRaises(RuntimeError):
                        licenses.debian_owners({Path('/usr/lib/example.so')})

    def test_multiple_paths_to_same_file_keep_both_owners(self):
        with tempfile.TemporaryDirectory() as directory:
            original = Path(directory, 'example.so.1')
            original.write_bytes(b'library')
            alias = Path(directory, 'example.so')
            alias.symlink_to(original)
            result = subprocess.CompletedProcess([], 0, f'example:amd64: {original}\n', '')
            with patch.object(licenses.subprocess, 'run', return_value=result):
                self.assertEqual(licenses.debian_owners({original, alias}),
                                 {original: 'example:amd64', alias: 'example:amd64'})

    def test_stale_notices_stop_packaging(self):
        with patch.object(licenses, 'rust_notices', return_value='changed lockfile/notices'):
            with self.assertRaisesRegex(RuntimeError, 'stale'):
                licenses.check_rust_notices()


if __name__ == '__main__':
    unittest.main()
