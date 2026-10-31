# Spotifast Lucid 0.5.0

Soloist can continue DJ and Smart Shuffle sessions started in official
Spotify and transferred through Connect. This update recognizes the reported
Smart Shuffle mode and protects current playback from an unsupported DJ start.

## New

- **Smart Shuffle status:** the main player bar's shuffle tooltip and
  accessibility label identify recommendation mode reported by Soloist.
  The button still toggles ordinary shuffle; it cannot enable Smart Shuffle.
- **Transfer guide:** the included installation guide explains starting DJ
  or Smart Shuffle in Spotify, transferring to Spotifast Soloist, and then
  fully closing Spotify. Both transferred modes were checked on Windows with
  Soloist 1.3.10.31 across a track transition with Spotify closed. DJ narration
  and long sessions have not been independently verified.

## Fixed

- **DJ starts preserve current music:** trying to load the special DJ
  playlist on the built-in player or this app's Soloist now shows useful
  instructions instead of replacing playback with an empty DJ context.

Native DJ launch, DJ voice requests, next-set controls and a Smart Shuffle
enable command remain unavailable. Account and regional availability depend
on Spotify. No new dependency, browser, service or experimental engine patch
is included. Both Raphael and Great Sage themes remain available.

Quit Spotifast with Ctrl+Q and run the new installer in the same folder,
or replace only the portable executable. Keep your existing profile private.
The Rust application retains its upstream Spotifast 0.14.2 version number;
0.5.0 identifies this fork's package.

## Thanks

**crmne and Spotifast contributors** (MIT), **sanoojes / Lucid** (AGPL-3.0
design reference), the project owner's **Raphael / TrueVision geometry and
slime icon**, **Spotify AB / Soloist**, and **Jordan Russell and Martijn Laan /
Inno Setup**. Artwork rights remain with their respective owners. This is an
unofficial fork; see `FORK.md` for full attribution.

[Installation and transfer guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md#dj-and-smart-shuffle-through-soloist)
· [Changes since 0.4.0](https://github.com/SxGoldenxS/Spotifast-Lucid/compare/lucid-v0.4.0...lucid-v0.5.0)
