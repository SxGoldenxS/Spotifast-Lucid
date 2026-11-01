#!/usr/bin/env python3
"""Release retries preserve downloads and do not hide incomplete releases."""

import copy
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch
import urllib.error


SPEC = importlib.util.spec_from_file_location("release_state", Path(__file__).with_name("lucid-release-state.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
RELEASE = {
    "tag_name": "lucid-v0.5.0",
    "draft": False,
    "assets": [
        {"name": name, "state": "uploaded", "size": 123, "digest": "sha256:" + "a" * 64}
        for name in sorted(MODULE.ASSETS)
    ],
}


class ReleaseStateTest(unittest.TestCase):
    def check(self, release, tag="lucid-v0.5.0"):
        response = io.BytesIO(json.dumps(release).encode())
        with patch.object(MODULE.urllib.request, "urlopen", return_value=response) as request:
            result = MODULE.already_published("owner/repo", tag, "test-token")
            self.assertEqual(request.call_args.kwargs["timeout"], 15)
            self.assertEqual(request.call_args.args[0].get_method(), "GET")
            return result

    def test_complete_release_is_reused(self):
        self.assertTrue(self.check(RELEASE))

    def test_modern_releases_require_the_signed_updater_assets(self):
        release = copy.deepcopy(RELEASE)
        release["tag_name"] = "v0.7.0"
        with self.assertRaises(ValueError):
            self.check(release, "v0.7.0")
        for name in ["spotifast-v0.7.0-x86_64-pc-windows-msvc.zip", "checksums.txt", "checksums.txt.sig"]:
            release["assets"].append({"name": name, "state": "uploaded", "size": 64, "digest": "sha256:" + "a" * 64})
        self.assertTrue(self.check(release, "v0.7.0"))

    def test_absent_release_can_be_built(self):
        error = urllib.error.HTTPError("https://api.github.com", 404, "Not Found", {}, None)
        with patch.object(MODULE.urllib.request, "urlopen", side_effect=error):
            self.assertFalse(MODULE.already_published("owner/repo", "lucid-v0.5.0", "test-token"))

    def test_access_and_server_errors_are_not_treated_as_absent(self):
        for code in [401, 403, 429, 500]:
            with self.subTest(code=code):
                error = urllib.error.HTTPError("https://api.github.com", code, "Failure", {}, None)
                with patch.object(MODULE.urllib.request, "urlopen", side_effect=error):
                    with self.assertRaises(RuntimeError):
                        MODULE.already_published("owner/repo", "lucid-v0.5.0", "test-token")

    def test_draft_and_wrong_tag_are_not_silently_accepted(self):
        for change in [{"draft": True}, {"tag_name": "lucid-v0.4.0"}]:
            with self.subTest(change=change):
                with self.assertRaises(ValueError):
                    self.check(RELEASE | change)

    def test_each_required_download_must_be_present(self):
        for index in range(len(RELEASE["assets"])):
            release = copy.deepcopy(RELEASE)
            del release["assets"][index]
            with self.assertRaises(ValueError):
                self.check(release)

    def test_unfinished_empty_or_unverified_downloads_are_not_accepted(self):
        for change in [{"state": "new"}, {"size": 0}, {"digest": None}, {"digest": "invalid"}]:
            release = copy.deepcopy(RELEASE)
            release["assets"][0].update(change)
            with self.subTest(change=change):
                with self.assertRaises(ValueError):
                    self.check(release)

    def test_invalid_version_never_sends_a_request(self):
        with patch.object(MODULE.urllib.request, "urlopen") as request:
            with self.assertRaises(ValueError):
                MODULE.already_published("owner/repo", "../other", "test-token")
            request.assert_not_called()


if __name__ == "__main__":
    unittest.main()
