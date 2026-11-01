#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile
from nacl.signing import SigningKey
from nacl.exceptions import BadSignatureError

SPEC = importlib.util.spec_from_file_location("package", Path(__file__).with_name("lucid-update-package.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class UpdatePackageTest(unittest.TestCase):
    @patch.object(MODULE, "current_version", return_value="0.7.0")
    def test_payload_contains_only_the_binary_and_signature_detects_tampering(self, _version):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            binary = root / "app.exe"
            binary.write_bytes(b"harmless fixture")
            for name in ["Spotifast-Lucid-Windows-x64.zip", "Spotifast-Lucid-Windows-x64-Setup.exe"]:
                (root / name).write_bytes(b"fixture download")
            (root / "private-settings.json").write_text("never package this")
            key = SigningKey.generate()
            archive = MODULE.package(binary, root, "0.7.0", key.encode().hex(), key.verify_key.encode().hex())
            with zipfile.ZipFile(archive) as zipped:
                self.assertEqual(zipped.namelist(), ["spotifast-v0.7.0-x86_64-pc-windows-msvc/spotifast.exe"])
                self.assertEqual(zipped.read(zipped.namelist()[0]), binary.read_bytes())
            manifest = (root / "checksums.txt").read_bytes()
            signature = (root / "checksums.txt.sig").read_bytes()
            self.assertEqual(len(signature), 64)
            key.verify_key.verify(manifest, signature)
            with self.assertRaises(BadSignatureError):
                key.verify_key.verify(manifest + b"tampered", signature)
            self.assertEqual(manifest, (root / "SHA256SUMS.txt").read_bytes())

    @patch.object(MODULE, "current_version", return_value="0.7.0")
    def test_wrong_publisher_key_and_version_are_rejected_before_writing(self, _version):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            key = SigningKey.generate()
            other = SigningKey.generate()
            for version, public in [("0.7.0", other.verify_key.encode().hex()), ("0.8.0", key.verify_key.encode().hex()), ("../escape", key.verify_key.encode().hex())]:
                with self.subTest(version=version), self.assertRaises(ValueError):
                    MODULE.package(root / "missing.exe", root, version, key.encode().hex(), public)
            self.assertFalse((root / "checksums.txt").exists())


if __name__ == "__main__":
    unittest.main()
