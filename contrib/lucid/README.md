# Lucid-inspired presentation for Spotifast

This local port adds rounded glass panels, a matching player and a softened
now-playing album background with short track transitions. It draws natively
with egui. It uses Spotifast's existing bounded artwork workers and respects
reduced motion and the existing **Colour from album art** setting.

`TrueVision.json` is an ordinary Spotifast palette: gold controls, ivory text,
dark blue surfaces and a teal fallback tint. It works with unmodified official
releases too. Copy it into the `themes` directory beside `settings.json`, run
`spotifast reload-themes`, then select **TrueVision** in Appearance.

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

## Scope and provenance

The design references [Lucid](https://github.com/sanoojes/spicetify-lucid).
This is an original native implementation; Lucid's React/CSS/shader code and
assets are not bundled. No browser, analytics or extra artwork service is
added. Spotify's existing covers supply the backdrop. Colours were mapped
from the local TrueVision overlay; its decorative artwork is not included.

This first port does not recreate Lucid's shader animation, every layout
option, or TrueVision's analytical-circle artwork. Queue, search, playback,
credentials and Spotify API behaviour remain owned by upstream Spotifast.
The newer upstream Soloist setup is retained. PC playback through librespot
can still fail if Spotify refuses audio keys; this presentation does not
change Spotify's authorization decisions.
