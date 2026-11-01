# Spotifast Lucid 0.7.0

Lucid now checks **our GitHub releases** on launch, using Spotifast's existing
native updater. Click the update badge or **Settings → About → Check for
updates**, then **Download update** and **Restart to update**. Downloads are
verified against our Ed25519 publisher signature and SHA-256 checksums. The
updater retains your profile, themes and pairing and rolls back if the new
app fails to start. Installation always waits for your click.

This release also integrates stable **Spotifast 0.15.0**, including the latest
Soloist source-volume fix and the small native Soloist audio adapter, while
retaining Raphael, Great Sage, DJ/Smart Shuffle transfer support and the native
timed-word/full-screen lyrics introduced in Lucid 0.6.0. Word timings still
require local TTML/enhanced LRC files; Spicy's hosted catalogue is not connected.

A one-time installer/portable upgrade is needed from 0.6.0 and earlier. Future
releases use standard `v0.7.0` style tags for native updater compatibility;
existing `lucid-v*` release links remain valid. The About section displays both
Lucid's package version and its integrated Spotifast version. No updater
service, browser engine or extra runtime is added.

Thanks to **crmne and Spotifast contributors** (MIT), **Spikerko / Spicy Lyrics**
and **sanoojes / Lucid** (AGPL-3.0 design references), the owner's **Raphael /
TrueVision artwork and slime icon**, **Spotify AB / Soloist**, and **Jordan
Russell and Martijn Laan / Inno Setup**. This is an independent fork. See
[FORK.md](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/FORK.md).

[Installation, updates and lyrics guide](https://github.com/SxGoldenxS/Spotifast-Lucid/blob/main/INSTALL-LUCID.md)
