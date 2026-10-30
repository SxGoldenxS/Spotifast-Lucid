# Spotifast Lucid / Raphael 0.2.0

Raphael's gold analytical arrays and teal glow now sit over the changing album
background. Native Rust/egui drawing keeps the app portable and lightweight.
This fork preview is based on Spotifast 0.14.2 plus the upstream source's Soloist
integration; the executable still reports its upstream version.

## New

- **Raphael background:** the original transparent TrueVision geometry,
  with two slow native rotations. It also works with album imagery disabled.
- **Bounded rendering:** three cached textures (about 8.4 MiB of GPU memory),
  with decoded RAM copies released after upload. Resizing reuses the cache.
  Rotation requests at most 12 fps and stops while paused, out of focus,
  saving energy, hidden or using reduced motion. No new dependencies,
  browser engine, network services or persistent settings.
- **Integrated window controls by default** for a fresh installation.
  Existing profiles retain their own setting.
- **Included installation guide:** sign-in, one-time Soloist pairing,
  troubleshooting discovery, updates and safe sharing.

## Install or update

Download the Windows x64 ZIP below, check `SHA256SUMS.txt` if desired, extract
it and run `spotifast.exe`. Follow the bundled `INSTALL-LUCID.md` or the
[online installation guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md).

For an existing installation, quit with Ctrl+Q and replace only the executable.
Keep `spotifast-data`; never replace it with another person's profile. Each
friend needs their own Premium account, Spotify app and personal Soloist key.
Official Spotify is needed for initial pairing only, then it can be closed.
Soloist is downloaded directly from Spotify and is not included in the ZIP.

## Credits

Thanks to **crmne and Spotifast contributors** (MIT), **sanoojes / Lucid**
(AGPL-3.0 design reference), and the project owner's **Raphael / TrueVision**
artwork and blue slime icon. Native presentation code is original Rust/egui;
artwork rights remain with the respective owners. Spotify Soloist belongs to
Spotify AB. This fork is unofficial. See `FORK.md` for full attribution.

The browser theme's individual shimmer/orbit effects and MilkDrop are not
included. This release uses a locally validated Windows build; see the release
validation note for the checks actually completed.
