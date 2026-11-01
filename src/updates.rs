//! Self-update from GitHub releases, through fastframe-update.
//!
//! The previous release's helper installs the first release built on the
//! crate, so what it relies on stays the same: `--version` prints
//! `<command> <version>`, the new app accepts `--update-receipt` and
//! `--update-error` (stripped by [`fastframe_update::intercept`] before the
//! argument parser), and the handoff and receipt files keep their format.

pub use fastframe_update::{
    CHECK_INTERVAL, DownloadState, Installation, Kind, Prepared, Release, Source, Unsupported,
    Updater,
};
use fastframe_update::{MacConfig, ReqwestTransport, UpdateConfig};

/// Fork package version, independent of the integrated upstream version.
pub const LUCID_VERSION: &str = "0.7.0";
pub const VERSION: &str = if cfg!(feature = "lucid") {
    LUCID_VERSION
} else {
    env!("CARGO_PKG_VERSION")
};
pub const RELEASES_URL: &str = if cfg!(feature = "lucid") {
    "https://github.com/SxGoldenxS/Spotifast-Lucid/releases"
} else {
    "https://github.com/crmne/spotifast/releases"
};

pub const CONFIG: UpdateConfig = if cfg!(feature = "lucid") {
    UpdateConfig {
        // Portable updates replace only the executable, including the legacy
        // spotifast-lucid.exe name, and retain the adjacent private profile.
        publisher_key: Some(include_str!("../assets/lucid-update-public-key.hex")),
        ..UpdateConfig::new(
            "SxGoldenxS/Spotifast-Lucid",
            "Spotifast Lucid",
            "spotifast",
            VERSION,
        )
    }
} else {
    UpdateConfig {
        macos: MacConfig {
            bundle_ids: &["rocks.spotifast.Spotifast"],
            executable_names: &["Spotifast"],
            legacy_bundle_names: &[],
        },
        // Releases publish an Ed25519 signature for checksums.txt. Require it
        // before downloading an update, using the release workflow's public key.
        publisher_key: Some(include_str!("../assets/update-public-key.hex")),
        // Releases build their AppImages before publishing, so the signed
        // checksums.txt lists them (#737): an AppImage replaces itself with the
        // next one.
        appimage: true,
        ..UpdateConfig::new(
            "crmne/spotifast",
            "Spotifast",
            "spotifast",
            env!("CARGO_PKG_VERSION"),
        )
    }
};

/// An updater on Spotifast's HTTP client, through the configured proxy.
pub fn updater(proxy: &crate::settings::ProxyConfig) -> anyhow::Result<Updater> {
    let builder = crate::http::blocking_builder(proxy).map_err(anyhow::Error::msg)?;
    Ok(Updater::new(CONFIG, ReqwestTransport::new(builder)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "lucid")]
    #[test]
    fn lucid_updates_use_the_fork_version_and_its_own_trust_root() {
        assert_eq!(CONFIG.repository, "SxGoldenxS/Spotifast-Lucid");
        assert_eq!(CONFIG.current_version, LUCID_VERSION);
        assert_ne!(
            CONFIG.publisher_key,
            Some(include_str!("../assets/update-public-key.hex"))
        );
        assert!(updater(&crate::settings::ProxyConfig::default()).is_ok());
        assert!(fastframe_update::is_newer("9999.0.0", VERSION));
        assert!(!fastframe_update::is_newer("0.0.0", VERSION));
        assert!(!fastframe_update::is_newer("lucid-v0.8.0", VERSION));
    }

    #[cfg(feature = "lucid")]
    #[test]
    fn fork_checks_only_its_own_github_and_ignores_legacy_or_unpublished_tags() {
        struct Listing(&'static str);
        impl fastframe_update::Transport for Listing {
            fn get(
                &self,
                request: &fastframe_update::Request<'_>,
            ) -> anyhow::Result<fastframe_update::Response> {
                assert_eq!(
                    request.url,
                    "https://api.github.com/repos/SxGoldenxS/Spotifast-Lucid/releases/latest"
                );
                Ok(fastframe_update::Response::ok(std::io::Cursor::new(
                    self.0.as_bytes(),
                )))
            }
        }
        let config = UpdateConfig {
            current_version: "0.6.0",
            ..CONFIG
        };
        let notice = Updater::new(config, Listing(r#"{"tag_name":"v0.7.0","html_url":"https://github.com/SxGoldenxS/Spotifast-Lucid/releases/tag/v0.7.0"}"#)).check().unwrap().unwrap();
        assert_eq!(notice.version, "0.7.0");
        for listing in [
            r#"{"tag_name":"lucid-v0.7.0","html_url":"https://example.invalid"}"#,
            r#"{"tag_name":"v0.7.0","draft":true,"html_url":"https://example.invalid"}"#,
            r#"{"tag_name":"v0.7.0","prerelease":true,"html_url":"https://example.invalid"}"#,
        ] {
            assert!(
                Updater::new(config, Listing(listing))
                    .check()
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[cfg(all(feature = "lucid", any(target_os = "windows", target_os = "linux")))]
    #[test]
    fn both_legacy_and_standard_portable_executable_names_can_update() {
        let root = std::env::temp_dir().join(format!("lucid-update-detect-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("spotifast-portable.txt"),
            "spotifast-portable-v1\n",
        )
        .unwrap();
        let updater = updater(&crate::settings::ProxyConfig::default()).unwrap();
        for name in ["spotifast.exe", "spotifast-lucid.exe"] {
            let executable = root.join(name);
            std::fs::write(&executable, b"harmless fixture").unwrap();
            assert_eq!(
                updater.installation_at(&executable).unwrap().kind,
                Kind::Portable
            );
        }
    }

    #[test]
    fn update_config_is_valid() {
        CONFIG.validate().unwrap();
        assert_eq!(CONFIG.current_version, VERSION);
    }

    #[test]
    fn updates_require_the_release_signing_key() {
        assert!(
            CONFIG.publisher_key.is_some(),
            "updates must require a publisher signature"
        );
        CONFIG.validate().unwrap();
    }

    /// An AppImage may replace itself only because the release workflow
    /// builds the AppImages into the signed checksums.txt;
    /// packaging/test-release-names.py checks that side.
    #[test]
    fn appimages_update_themselves() {
        const { assert!(CONFIG.appimage != cfg!(feature = "lucid")) };
    }
}
