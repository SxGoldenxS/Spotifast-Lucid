#!/usr/bin/env python3
"""Build fastframe's portable payload and sign its exact checksum manifest.

PyNaCl is build tooling only. The client already verifies Ed25519 through
fastframe-update; no runtime dependency or new update engine is added.
"""
import argparse
import hashlib
import os
from pathlib import Path
import re
import zipfile

from nacl.signing import SigningKey, VerifyKey

ROOT = Path(__file__).resolve().parents[2]


def current_version():
    source = (ROOT / "src/updates.rs").read_text(encoding="utf-8")
    match = re.search(r'pub const LUCID_VERSION: &str = "([0-9.]+)";', source)
    if not match:
        raise ValueError("Missing Lucid version")
    return match[1]


def package(binary, directory, version, seed, public_key):
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Invalid package version")
    if version != current_version():
        raise ValueError("Package version must match the Lucid binary version")
    try:
        key = SigningKey(bytes.fromhex(seed))
    except (ValueError, TypeError):
        raise ValueError("A valid publisher signing key is required") from None
    if key.verify_key.encode().hex() != public_key.strip():
        raise ValueError("Publisher key does not match the committed verification key")
    stem = f"spotifast-v{version}-x86_64-pc-windows-msvc"
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / f"{stem}.zip"
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        zipped.write(binary, f"{stem}/spotifast.exe")
    names = [archive.name, "Spotifast-Lucid-Windows-x64.zip", "Spotifast-Lucid-Windows-x64-Setup.exe"]
    entries = []
    for name in sorted(names):
        with (directory / name).open("rb") as payload:
            entries.append(f"{hashlib.file_digest(payload, 'sha256').hexdigest()}  {name}\n")
    manifest = "".join(entries).encode("ascii")
    signature = key.sign(manifest).signature
    VerifyKey(bytes.fromhex(public_key.strip())).verify(manifest, signature)
    (directory / "checksums.txt").write_bytes(manifest)
    (directory / "checksums.txt.sig").write_bytes(signature)
    (directory / "SHA256SUMS.txt").write_bytes(manifest)
    return archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    package(args.binary, args.directory, args.version,
            os.environ.get("LUCID_UPDATE_SIGNING_KEY", ""),
            (ROOT / "assets/lucid-update-public-key.hex").read_text(encoding="ascii"))
    print("Portable update payload and publisher signature verified.")


if __name__ == "__main__":
    main()
