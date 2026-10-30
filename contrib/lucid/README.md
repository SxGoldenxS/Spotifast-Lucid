# Lucid-inspired presentation for Spotifast

This local port adds rounded glass panels, a matching player, Raphael's gold
arrays and teal glow, and a softened now-playing album background with short
track transitions. It draws natively
with egui. It uses Spotifast's existing bounded artwork workers and respects
reduced motion and the existing **Colour from album art** setting.

`TrueVision.json` is an ordinary Spotifast palette: gold controls, ivory text,
dark blue surfaces and a teal fallback tint. It works with unmodified official
releases too. Copy it into the `themes` directory beside `settings.json`, run
`spotifast reload-themes`, then select **TrueVision** in Appearance.
The palette changes colours only on stock Spotifast. Raphael's decorative
background requires this fork built with `--features lucid`.

`GreatSage.json` is the separate blue variant: navy surfaces, azure controls,
ice-white text and cyan/blue arrays. Select **GreatSage** in Appearance. It
shares Raphael's geometry, recoloured once when selected; no duplicate SVGs,
browser, worker, dependency or persistent setting is added. Switching themes
releases the previous array textures, keeping only one 8.4 MiB set on the GPU.

## Build and update on Windows

Install the pinned Rust toolchain and Visual Studio Build Tools' C++ workload.
From this checkout:

```powershell
./contrib/lucid/build.ps1 -Demo
./dist/Spotifast-Lucid/spotifast.exe --demo
```

The first build excludes MilkDrop's additional CMake/LLVM/vcpkg prerequisites.
Add `-MilkDrop` when those upstream prerequisites are installed. A production
build omits `-Demo`. No new dependencies or persistent settings are added.

Commit the local port before updating, then run:

```powershell
./contrib/lucid/update.ps1 -Demo
```

The update script requires a clean checkout and the official `upstream` remote.
It imports new upstream commits on a separate candidate branch, preserving
upstream authorship and the fork's published history. `upstream.txt` records
the last imported upstream commit. Tests and Clippy must pass before the
original branch advances by fast-forward. No force-push is needed. Conflicts
stop for review; use `git cherry-pick --abort` during a conflict, then switch
back to the original branch. A failed build stays on the candidate branch for
repair, with the original branch retained. Builds do not replace a running
Lucid executable; the last packaged
executable is retained as `spotifast.exe.previous`. Existing portable data is
preserved. A failed build leaves the installed executable untouched.

The custom build blocks the stock binary updater, which would remove this
port. Update this checkout instead. Upstream builds without `--features lucid`
retain their original UI and updater.

## Packaging the lightweight installer

With Inno Setup 6.3+ available on the build machine, after a production build:

```powershell
./contrib/lucid/package.ps1 -Version 0.4.0
```

This creates a clean portable ZIP, a native per-user Setup.exe and checksums
under `dist/Installers`. The installer carries the same self-contained Rust
binary. It writes no shared Spotify protocol associations, installs no service
or prerequisites, and uses the existing portable profile layout in its own
install folder. A stable AppId supports later upgrades. Existing settings and
palettes are not overwritten; all profile files survive uninstall. A running
copy in the selected folder blocks update/uninstall rather than being killed.
An original Spotifast installer folder is refused to keep profiles separate.

`windows-installer.iss` lists every bundled file explicitly. No configured
profile or recursive payload directory can enter the installer. Native tests
in `test-installer.py` compile this same script with isolated app identities
and harmless C payloads, checking install, update, shortcuts, lock handling,
profile preservation, uninstall and reinstall. The Windows release workflow
runs them before packaging both downloads. The bootstrap workflow packages
the unchanged, checksum-verified 0.3.0 Rust payload for installer-only 0.4.0.

## Scope and provenance

The design references [Lucid](https://github.com/sanoojes/spicetify-lucid).
This is an original native implementation; Lucid's React/CSS/shader code and
assets are not bundled. No browser, analytics or extra artwork service is
added. Spotify's existing covers supply the backdrop. Colours were mapped
from the local TrueVision overlay. `raphael/*.svg` contains its transparent
geometric artwork, supplied by the project owner. Original artwork source and
licence were not provided; rights remain with the respective owners, separate
from the code's MIT licence. No personal icon, account details, scripts,
external references or embedded browser animations are in these SVGs.

The stock SVG loader rasterizes each decoration once at a fixed size: primary
1280 square, secondary 640 square, glow 256 by 512. GPU textures total 8.4 MiB;
the existing `images::forget_decoded` helper releases decoded RAM copies after
upload. Resizing and DPI changes reuse those same textures.
Three textured quads draw the scene behind page content without
changing layout or input. Rotation is capped at 12 fps, uses the existing
power budget, stops while paused/out of focus/saving energy/reducing motion,
and never creates an independent timer or worker. The artwork follows the
selected `TrueVision.json` theme and remains visible without album imagery.

This port does not recreate Lucid's browser shader animation, every layout
option, or every individual shimmer/orbit in the original SVG. Queue, search, playback,
credentials and Spotify API behaviour remain owned by upstream Spotifast.
The newer upstream Soloist setup is retained. PC playback through librespot
can still fail if Spotify refuses audio keys; this presentation does not
change Spotify's authorization decisions.
