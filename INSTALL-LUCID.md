# Install Spotifast Lucid / Raphael on Windows

1. Download **Spotifast-Lucid-Windows-x64.zip** from
   [the latest release](https://github.com/SxGoldenxS/Spotifast-Lucid/releases/latest).
   Optionally compare its SHA-256 with the supplied `SHA256SUMS.txt`.
2. Extract the entire ZIP to a folder you own. Run `spotifast.exe` inside it.
   This is an unsigned Windows x64 portable app, with no installer or admin requirement.
3. Follow Spotifast's sign-in instructions to create your own free Spotify
   developer app. Enable Web API and set its redirect URI to
   `http://127.0.0.1:8989/login`. Enter its **Client ID** in Spotifast and sign in.
4. Choose **Set up Soloist**. Obtain your own personal key from the Spotify
   page linked in setup, then choose **Download and enable Soloist**.
   A Spotify Premium account is required.
5. Once, open official Spotify on the same account and network. In its device
   picker, select the exact **Spotifast Soloist (...)** name shown in setup.
   If the name is missing, fully quit and reopen official Spotify to refresh
   discovery. Return to Spotifast and choose **Start listening**.

You can now close official Spotify. Spotifast starts its managed Soloist player
automatically and restores the saved pairing on later launches. Soloist comes
directly from Spotify; it is not bundled or redistributed in this ZIP.

## Theme and window

The ZIP selects **TrueVision**, the gold/ivory Raphael theme, by default.
Its transparent gold arrays, teal glow and changing album background appear
on the main page. **Settings → Appearance → Colour from album art** controls
the album imagery; the decorative arrays also work with that setting off.
Choose another theme to remove the Raphael artwork.
For the separate blue **Great Sage** variant, choose **GreatSage** in
**Settings → Appearance → Theme**. It has deep navy panels, azure controls,
ice-white text and blue/cyan arrays. Choose **TrueVision** to return to gold.
Both keep the same album backgrounds and slow rotation behaviour.
Use this fork for the decorative background; copying its palette into stock
Spotifast changes colours only.

Fresh release ZIPs also select integrated window controls. Use **Settings →
Appearance → Custom title bar** to change that choice. Drag empty space in the
top header to move the window. The choice is
saved. Search settings for the setting's name if it is not visible.

The arrays rotate slowly during playback, at most 12 frames per second. They
stay still while paused, out of focus, saving energy or using the system's
reduced-motion setting. Artwork is bundled locally and cached at fixed sizes;
resizing does not create a growing set of textures. No browser or new service
is added.

## Update or share

Quit Spotifast completely with **Ctrl+Q**. Back up the old executable and
replace only `spotifast.exe` with the one from the new ZIP. Keep your existing
`spotifast-data` folder so settings, sign-in and pairing survive. Do not copy
fresh default settings over an existing profile.
Copy any new palette files you do not already have, such as `GreatSage.json`,
from the ZIP's `spotifast-data/config/themes` into your own themes folder.
Then reopen Spotifast and select the new theme in Appearance.

Send friends the release link or clean release ZIP. **Never share your running
app folder, `spotifast-data`, Client ID, Soloist key or credentials.** Each person
sets up their own account. The stock Spotifast binary updater is disabled in
this fork so it cannot replace the port.

## Credits

Built on [Spotifast by crmne and contributors](https://github.com/crmne/spotifast)
(MIT), with native Rust/egui presentation inspired by
[Lucid by sanoojes](https://github.com/sanoojes/spicetify-lucid) (AGPL-3.0).
Raphael / TrueVision geometric artwork and the blue slime icon were supplied
by the project owner; their artwork rights remain with the respective owners.
Spotify Soloist is Spotify AB's separate software. This is an unofficial fork.
See [FORK.md](FORK.md) for full scope and attribution.
