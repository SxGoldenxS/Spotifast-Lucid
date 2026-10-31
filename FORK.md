# Spotifast Lucid

An unofficial Windows preview maintained by [SxGoldenxS](https://github.com/SxGoldenxS).

## Download and run

For easy setup, run the [Windows installer](https://github.com/SxGoldenxS/Spotifast-Lucid/releases/latest/download/Spotifast-Lucid-Windows-x64-Setup.exe).
It installs for the current user, adds a Start menu entry and offers a desktop
shortcut. Updates preserve the profile and customised palettes. Uninstall
removes program files and shortcuts but keeps the profile for reinstall.

Alternatively, download the Windows ZIP from [Releases](https://github.com/SxGoldenxS/Spotifast-Lucid/releases/latest),
check its SHA-256 against `SHA256SUMS.txt`, extract it into a folder you own,
and run `spotifast.exe`. Windows may identify this unsigned custom build as an
unknown publisher. No installer or administrator access is needed.
See the [installation guide](INSTALL-LUCID.md) for sign-in, one-time Soloist
pairing, the Raphael background, window controls and safe updates.
The package includes both gold **TrueVision / Raphael** and blue **GreatSage**.
Switch between them in **Settings → Appearance → Theme**.

This fork keeps its settings in `spotifast-data` beside the executable. Keep
that folder private. Do not send your configured folder to friends: send the
GitHub release ZIP instead. Each person supplies their own account and keys.

1. Create your own Web API app in Spotify's developer dashboard, following
   Spotifast's sign-in instructions. Use `http://127.0.0.1:8989/login` as its
   redirect URI, enable Web API, and enter its Client ID in Spotifast.
2. Choose **Set up Soloist**, obtain your personal key through Spotify's
   linked page, and enter it directly into Spotifast.
3. Download and enable Soloist. In Spotify's official app on the same network
   and account, choose the exact **Spotifast Soloist (...)** device shown in
   setup. Return and choose **Start listening** after verification.

Soloist is Spotify's own separately downloaded player. It is not bundled in
this repository or release. Its saved key is protected by the operating system.
Premium is required. See the [upstream guide](https://spotifast.rocks/soloist/).
For DJ and Smart Shuffle, use the
[start-and-transfer workflow](INSTALL-LUCID.md#dj-and-smart-shuffle-through-soloist).
This fork recognizes Soloist's reported Smart Shuffle mode and prevents an
unsupported DJ playlist load from replacing current playback. It does not
generate DJ sessions or add a Smart Shuffle enable command.
If the built-in librespot player reports that Spotify refused an audio key,
an official Spotify device or Soloist is the alternative; reinstalling or
changing theme settings does not fix that Spotify-side refusal.

## Updating

For a downloaded copy, close Spotifast completely, retain a backup of the old
executable, and replace only `spotifast.exe` from the new release. Keep your
existing `spotifast-data` folder. Never overwrite it with another person's.
An installed copy can instead rerun the newer installer in the same folder.
The official binary updater is blocked in Lucid builds so it cannot silently
remove the port. Download new Lucid versions from this fork's Releases page.

For source updates, clone this fork and add the original repository:

```powershell
git remote add upstream https://github.com/crmne/spotifast.git
./contrib/lucid/update.ps1
```

The script requires a clean checkout, imports new upstream commits on a
separate candidate branch, and runs the build checks before fast-forwarding
your original branch. Published history and upstream author credits are
preserved; push normally without a force-push. Resolve conflicts before
publishing; use `git cherry-pick --abort` during a conflict and switch back
to your original branch if needed. See
[development details](contrib/lucid/README.md). The custom code is isolated in
`src/ui/lucid.rs`; the remaining integration points are frame constructors,
the existing icon renderer, Windows resources and the updater guard.

The **Lucid Windows** GitHub workflow builds and tests the preview. Publishing
a `lucid-v*` tag creates a Windows ZIP and checksums from explicit clean files.
Ordinary source builds omit MilkDrop; pass `-MilkDrop` only after installing
the upstream CMake, LLVM and vcpkg requirements.

## Credits and licences

- **Spotifast:** [crmne and all upstream contributors](https://github.com/crmne/spotifast),
  MIT. Upstream history, copyright, licence and dependency acknowledgements
  are retained. This fork is independent and is not an official upstream release.
- **Lucid design reference:** [sanoojes/spicetify-lucid](https://github.com/sanoojes/spicetify-lucid),
  published under AGPL-3.0. This port uses original Rust/egui drawing code;
  Lucid's CSS, React, shaders and artwork are not copied or bundled.
- **Raphael / TrueVision:** palette and geometric array artwork adapted from
  the project owner's existing TrueVision/Lucid customization. The original
  transparent SVG geometry is bundled locally in `contrib/lucid/raphael`.
  Original artwork source/licence was not provided; artwork rights remain
  with the respective owners and are not offered under the code's MIT licence.
- **Blue slime icon:** supplied by the project owner and prepared with a
  transparent background. Original artist/source was not provided. Rights to
  the supplied illustration and depicted character remain with their owners;
  the icon is not offered under the code's MIT licence.
- **Spicy Lyrics design reference:** [Spikerko/spicy-lyrics](https://github.com/Spikerko/spicy-lyrics),
  AGPL-3.0. Native timed-word rendering and TTML/enhanced-LRC parsing are an
  independent Rust implementation. No extension TypeScript/CSS is copied or
  bundled. Spicy's hosted catalogue and services have separate access terms
  and are not included. See the [lyrics guide](INSTALL-LUCID.md#word-highlighting-and-full-screen-lyrics).
- **Dependencies:** librespot, egui, fastframe, Inter, Lucide and the other
  upstream dependencies retain their respective licences and acknowledgements.
- **Spotify/Soloist:** Spotify AB. This fork is not affiliated with or
  endorsed by Spotify. Spotify's player is downloaded directly from Spotify.
- **Installer engine:** [Inno Setup](https://jrsoftware.org/isinfo.php),
  Jordan Russell and Martijn Laan. It runs only during setup/uninstall and
  adds no service, browser or runtime dependency to the Rust application.

## Preview scope

Rounded glass panels, a matching player, softened track-derived artwork,
short crossfades, normal custom JSON palettes, and matching app/tray icons.
The TrueVision theme also draws Raphael's gold arrays and teal glow over the
album background. Two slow native rotations replace the original browser SVG
animation; reduced motion, pause and the existing power budget stop rotation.
Three fixed-size cached textures use about 8.4 MiB of GPU memory in total,
independent of window resizing. Decoded pixel copies are released after upload.
Each frame adds only three quads.
Great Sage recolours the same geometry once on selection and releases the
inactive theme's textures, retaining the same memory and drawing budget.
Existing account, queue and playback logic remains upstream-owned. This first
port does not reproduce every Lucid setting or its browser shader and shimmer
animations. MilkDrop is omitted from the Windows package.
