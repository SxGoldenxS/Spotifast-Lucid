#!/usr/bin/env python3
"""Reuse a complete published Lucid release without rebuilding or replacing it."""

import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request


ASSETS = {
    "Spotifast-Lucid-Windows-x64-Setup.exe",
    "Spotifast-Lucid-Windows-x64.zip",
    "SHA256SUMS.txt",
}


def already_published(repo, tag, token):
    if not re.fullmatch(r"lucid-v\d+\.\d+\.\d+", tag):
        raise ValueError("Expected a Lucid version tag such as lucid-v0.5.0")
    if not repo or not token:
        raise ValueError("GitHub repository and token are required for release checks")
    url = f"https://api.github.com/repos/{repo}/releases/tags/{urllib.parse.quote(tag, safe='')}"
    request = urllib.request.Request(url, headers={
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    })
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            release = json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise RuntimeError(f"GitHub release check failed (HTTP {error.code})") from None
    if release.get("tag_name") != tag or release.get("draft") is not False:
        raise ValueError("Existing release is not published for this tag; inspect it before continuing")
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    if any(
        name not in assets
        or assets[name].get("state") != "uploaded"
        or assets[name].get("size", 0) <= 0
        or not re.fullmatch(r"sha256:[0-9a-f]{64}", assets[name].get("digest") or "")
        for name in ASSETS
    ):
        raise ValueError("Published release is missing verified downloads; inspect it before continuing")
    return True


def main():
    published = already_published(
        os.environ.get("GITHUB_REPOSITORY", ""),
        os.environ.get("RELEASE_TAG", ""),
        os.environ.get("GH_TOKEN", ""),
    )
    value = str(published).lower()
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a", encoding="utf-8") as stream:
            stream.write(f"published={value}\n")
    if published:
        print("Release already published. Keeping its existing downloads.", file=sys.stderr)
    print(value)


if __name__ == "__main__":
    main()
