# Spotifast Lucid 0.4.0

An easy Windows installer joins the portable ZIP. It installs the same
lightweight Rust app with both Raphael and Great Sage themes.

## New

- **Easy setup:** run `Spotifast-Lucid-Windows-x64-Setup.exe`, choose a folder
  and optional desktop shortcut, then launch from the Start menu. No admin
  rights, extra runtime, service or browser engine required.
- **Simple upgrades:** fully quit with Ctrl+Q and rerun the newer installer
  in the same folder. Settings, sign-in, pairing and edited palettes stay.
  A running or locked copy blocks replacement before program files change.
- **Clean uninstall:** Windows Settings → Apps removes program files and
  installer-created shortcuts, keeping the profile for reinstall.
- **Included guides and credits:** installation help, personal Spotify setup,
  themes, updates and safe sharing. Friends still supply their own account
  and keys. Soloist is downloaded directly from Spotify and is not bundled.

For a current portable copy, choose its existing folder in Setup to retain
its `spotifast-data` profile. The original Spotifast installer's folder is
refused to keep the independent installations separate. Spotify link
associations and Windows startup settings are not changed.

This installer-only release uses the unchanged, checksum-verified 0.3.0 Rust
application, which retains its upstream Spotifast 0.14.2 version number.
The source's Windows release workflow now builds the installer automatically
for future updates. The portable ZIP remains available.

## Thanks

**crmne and Spotifast contributors** (MIT), **sanoojes / Lucid** (AGPL-3.0
design reference), the project owner's **Raphael / TrueVision geometry and
slime icon**, **Spotify AB / Soloist**, and **Jordan Russell and Martijn Laan /
Inno Setup**. Artwork rights remain with their respective owners. This is an
unofficial fork; see the included `FORK.md` for full attribution.

[Installation guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md)
· [Changes since 0.3.0](https://github.com/SxGoldenxS/Spotifast-Lucid/compare/lucid-v0.3.0...lucid-v0.4.0)
