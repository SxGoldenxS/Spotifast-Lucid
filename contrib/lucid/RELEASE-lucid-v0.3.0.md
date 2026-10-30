# Spotifast Lucid / Great Sage 0.3.0

A separate blue Great Sage theme joins the gold Raphael theme. Choose
**GreatSage** under **Settings → Appearance → Theme** for deep navy panels,
azure controls, ice-white text and blue/cyan analytical arrays. **TrueVision**
remains available for the original gold look.

The themes share one source of geometric artwork. Blue colours are applied
once when selected, and switching themes releases the previous GPU textures.
Only one 8.4 MiB array texture set stays active. Both themes use the same three
textured quads and slow rotation, stopping while paused, unfocused, saving
energy or using reduced motion. No dependencies, browser or service added.

## Install or update

Extract the Windows x64 ZIP and follow the included **INSTALL-LUCID.md**, or
read the [online guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md).
For an existing installation, fully quit Spotifast with Ctrl+Q, replace only
the executable, and copy **GreatSage.json** from the ZIP's
`spotifast-data/config/themes` into your own themes folder. Keep your existing
settings and profile. Reopen Spotifast and select GreatSage in Appearance.

Spotify Premium, your own Spotify developer app and a personal Soloist key
are required. Official Spotify is needed for initial Soloist pairing only.
Never share a configured profile or keys; send friends this clean release ZIP.

## Credits and scope

Built on **Spotifast by crmne and contributors** (MIT), with native Rust/egui
presentation inspired by **sanoojes / Lucid** (AGPL-3.0 design reference).
Raphael / TrueVision geometry and the slime icon were supplied by the project
owner; Great Sage adapts their colours. Artwork rights remain with their
respective owners. Spotify Soloist is Spotify AB's separately downloaded
software. This fork is unofficial; see **FORK.md** for full attribution.

This is a blue colour variant of the current native animation, not a new port
of the browser theme's individual shimmer/orbit effects. MilkDrop is omitted.
The binary retains its upstream Spotifast 0.14.2 version number.
