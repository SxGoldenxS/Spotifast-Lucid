"""Native install/update/uninstall checks with an isolated identity and fake app.

Run on a Windows build runner after loading the MSVC environment:
  python contrib/lucid/test-installer.py path/to/ISCC.exe
No Spotify connection, real profile or production shortcut is used.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
ISCC = sys.argv.pop(1) if len(sys.argv) > 1 else shutil.which("iscc")
FLAGS = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/SP-"]


@unittest.skipUnless(sys.platform == "win32" and ISCC, "requires Windows and Inno Setup")
class InstallerTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import winreg
        cls.winreg = winreg
        cache = REPO / ".cache"
        cache.mkdir(exist_ok=True)
        cls.temp = tempfile.TemporaryDirectory(prefix="installer-test-", dir=cache)
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        cls.identity = "SpotifastLucidTest-" + uuid.uuid4().hex
        cls.shortcut = Path(os.environ["APPDATA"]) / "Microsoft/Windows/Start Menu/Programs" / (cls.identity + ".lnk")
        cls.uninstall_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\" + cls.identity + "_is1"
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders") as key:
            cls.desktop = Path(os.path.expandvars(winreg.QueryValueEx(key, "Desktop")[0])) / (cls.identity + ".lnk")
        source = cls.root / "fixture.c"
        source.write_text(r'''
#include <windows.h>
#if FIXTURE_VARIANT == 1
#define FIXTURE_TEXT "fixture-one"
#else
#define FIXTURE_TEXT "fixture-two"
#endif
int wmain(int argc, wchar_t **argv) {
    if (argc < 2) return 0;
    HANDLE ready = CreateFileW(argv[1], GENERIC_WRITE, FILE_SHARE_READ, 0,
                              CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (ready == INVALID_HANDLE_VALUE) return 2;
    DWORD written;
    WriteFile(ready, FIXTURE_TEXT, sizeof(FIXTURE_TEXT), &written, 0);
    CloseHandle(ready);
    for (;;) Sleep(1000);
}
''')
        cls.payloads, cls.setups = [], []
        for variant in (1, 2):
            payload = cls.root / f"payload-{variant}.exe"
            subprocess.run(["cl", "/nologo", "/O1", "/MT", f"/DFIXTURE_VARIANT={variant}",
                            f"/Fe:{payload}", f"/Fo:{cls.root / f'fixture-{variant}.obj'}", str(source)],
                           check=True, timeout=90)
            version = f"0.4.{variant - 1}"
            output = cls.root / f"variant-{variant}"
            output.mkdir()
            subprocess.run([ISCC, "/Q", f"/DVersion={version}", f"/DBinary={payload}",
                            f"/DOutputDir={output}", f"/DProductId={cls.identity}",
                            f"/DProductName={cls.identity}", str(ROOT / "windows-installer.iss")],
                           check=True, timeout=90)
            cls.payloads.append(payload)
            cls.setups.append(output / "Spotifast-Lucid-Windows-x64-Setup.exe")

    def setUp(self):
        self.app = self.root / ("install with spaces é-" + uuid.uuid4().hex)
        self.processes = []
        self.assertEqual(self.install(), 0)

    def tearDown(self):
        for process in self.processes:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=10)
        if (self.app / "unins000.exe").exists():
            self.assertEqual(self.uninstall(), 0)

    def install(self, variant=0, target=None, extra=()):
        return subprocess.run([str(self.setups[variant]), *FLAGS, f"/DIR={target or self.app}", *extra], timeout=90).returncode

    def uninstall(self):
        result = subprocess.run([str(self.app / "unins000.exe"), *FLAGS], timeout=90).returncode
        if result == 0:
            deadline = time.monotonic() + 10
            while (self.app / "unins000.exe").exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertFalse((self.app / "unins000.exe").exists())
        return result

    def run_payload(self, path):
        ready = self.root / (uuid.uuid4().hex + ".ready")
        process = subprocess.Popen([str(path), str(ready)], stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL, creationflags=subprocess.CREATE_NO_WINDOW)
        self.processes.append(process)
        deadline = time.monotonic() + 10
        while not ready.exists() and time.monotonic() < deadline:
            self.assertIsNone(process.poll())
            time.sleep(0.05)
        self.assertTrue(ready.exists())
        return process

    def snapshot(self):
        return {str(p.relative_to(self.app)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in self.app.rglob("*") if p.is_file()}

    def personalise(self):
        config = self.app / "spotifast-data/config"
        (config / "settings.json").write_text('{"custom_theme":"GreatSage.json","fixture":true}')
        (config / "themes/GreatSage.json").write_text('{"fixture":"custom palette"}')
        state = self.app / "spotifast-data/state/soloist/accounts/fixture"
        state.mkdir(parents=True)
        (state / "pairing.fixture").write_bytes(b"dummy pairing: not a credential")
        return {str(p.relative_to(self.app)): p.read_bytes()
                for p in (self.app / "spotifast-data").rglob("*") if p.is_file()}

    def test_fresh_install_has_both_themes_shortcut_and_per_user_uninstaller(self):
        self.assertEqual((self.app / "spotifast.exe").read_bytes(), self.payloads[0].read_bytes())
        self.assertEqual((self.app / "spotifast-portable.txt").read_text().strip(), "spotifast-portable-v1")
        self.assertFalse((self.app / "spotifast-installer.txt").exists())
        config = self.app / "spotifast-data/config"
        self.assertEqual(json.loads((config / "settings.json").read_text())["custom_theme"], "TrueVision.json")
        for name in ("TrueVision.json", "GreatSage.json"):
            self.assertIn("accent", json.loads((config / "themes" / name).read_text())["colors"])
        self.assertTrue(self.shortcut.exists())
        self.assertFalse(self.desktop.exists())
        with self.winreg.OpenKey(self.winreg.HKEY_CURRENT_USER, self.uninstall_key) as key:
            self.assertIn(str(self.app), self.winreg.QueryValueEx(key, "InstallLocation")[0])

    def test_desktop_shortcut_is_optional_and_removed_on_uninstall(self):
        self.assertEqual(self.install(extra=("/TASKS=desktopicon",)), 0)
        self.assertTrue(self.desktop.exists())
        self.assertEqual(self.uninstall(), 0)
        self.assertFalse(self.desktop.exists())

    def test_upgrade_and_reinstall_preserve_the_entire_profile(self):
        before = self.personalise()
        self.assertEqual(self.install(1), 0)
        self.assertEqual((self.app / "spotifast.exe").read_bytes(), self.payloads[1].read_bytes())
        self.assertEqual(self.uninstall(), 0)
        self.assertFalse((self.app / "spotifast.exe").exists())
        self.assertFalse(self.shortcut.exists())
        with self.assertRaises(FileNotFoundError):
            self.winreg.OpenKey(self.winreg.HKEY_CURRENT_USER, self.uninstall_key)
        for name, contents in before.items():
            self.assertEqual((self.app / name).read_bytes(), contents)
        self.assertEqual(self.install(1), 0)
        for name, contents in before.items():
            self.assertEqual((self.app / name).read_bytes(), contents)

    def test_running_copy_blocks_upgrade_and_uninstall_without_any_changes(self):
        process = self.run_payload(self.app / "spotifast.exe")
        before = self.snapshot()
        self.assertNotEqual(self.install(1), 0)
        self.assertEqual(self.snapshot(), before)
        self.assertNotEqual(self.uninstall(), 0)
        self.assertEqual(self.snapshot(), before)
        self.assertIsNone(process.poll())

    def test_legacy_named_copy_also_blocks_replacement(self):
        legacy = self.app / "spotifast-lucid.exe"
        shutil.copyfile(self.payloads[0], legacy)
        process = self.run_payload(legacy)
        before = self.snapshot()
        self.assertNotEqual(self.install(1), 0)
        self.assertEqual(self.snapshot(), before)
        self.assertIsNone(process.poll())

    def test_another_copy_elsewhere_is_not_stopped_or_treated_as_a_lock(self):
        elsewhere = self.root / ("elsewhere-" + uuid.uuid4().hex)
        elsewhere.mkdir()
        payload = elsewhere / "spotifast.exe"
        shutil.copyfile(self.payloads[0], payload)
        process = self.run_payload(payload)
        self.assertEqual(self.install(1), 0)
        self.assertEqual(self.uninstall(), 0)
        self.assertIsNone(process.poll())

    def test_original_installer_folder_is_left_untouched(self):
        foreign = self.root / ("original-" + uuid.uuid4().hex)
        foreign.mkdir()
        marker = foreign / "spotifast-installer.txt"
        marker.write_text("spotifast-installer-v1")
        self.assertNotEqual(self.install(1, foreign), 0)
        self.assertEqual([p.name for p in foreign.iterdir()], [marker.name])


if __name__ == "__main__":
    unittest.main(verbosity=2)
