# Spotifast Lucid 0.6.0

Native word highlighting is now available in the lyrics panel and full-screen
lyrics when a song has an imported TTML or enhanced LRC file with real timings.
Use **Lyrics options** (ellipsis) in the panel, or right-click **Lyrics** in
full screen, then choose **Import lyrics file...**. Your theme accent follows
the words. Existing Spotify/LRCLIB lyrics keep their line/static fallback.

Imports are private, stored per track and retained across upgrades. The
original file stays unchanged. **Use online lyrics** restores normal lookup.
Invalid or cancelled imports preserve the current lyrics. This first stage
supports lead vocals; Spicy's hosted catalogue, translations and duet layouts
are not included. No Spotify token is sent to Spicy Lyrics.

The implementation remains Rust/egui. It reuses the existing XML dependency
and text shaping, avoids copying the document every frame and caps active
word animation at 30 fps. Paused/hidden views stop word animation; energy
saving and reduced motion use discrete highlights.

Quit Spotifast with Ctrl+Q and rerun Setup in the same folder, or replace only
the portable executable. Keep your existing `spotifast-data` folder private.
The upstream app version remains 0.14.2; 0.6.0 identifies this fork's package.

Thanks to **crmne and Spotifast contributors** (MIT), **Spikerko / Spicy Lyrics**
(AGPL-3.0 design reference), **sanoojes / Lucid** (AGPL-3.0 design reference),
the owner's **Raphael / TrueVision artwork and slime icon**, **Spotify AB /
Soloist**, and **Jordan Russell and Martijn Laan / Inno Setup**. Timed lyrics
use original Rust code; no Spicy extension source is bundled. Artwork and
services retain their respective owners' rights. See `FORK.md`.

[Installation and lyrics guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md#word-highlighting-and-full-screen-lyrics)
