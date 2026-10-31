//! Bounded local TTML and enhanced LRC imports, independent of hosted services.
use super::{Line, Lyrics, Word};
use anyhow::{Context, Result, ensure};
use roxmltree::Node;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_TIME: u32 = 24 * 60 * 60 * 1_000;
const MAX_LINES: usize = 2_000;
const MAX_WORDS: usize = 20_000;

fn bounded_text(path: &Path) -> Result<String> {
    let metadata = std::fs::metadata(path)?;
    ensure!(
        metadata.is_file() && metadata.len() <= MAX_BYTES as u64,
        "Choose a regular lyrics file no larger than 2 MiB"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= MAX_BYTES, "Lyrics file exceeds 2 MiB");
    String::from_utf8(bytes).context("Lyrics must be UTF-8 text")
}

pub fn override_path(directory: &Path, uri: &str) -> PathBuf {
    directory.join(format!("{:x}.json", Sha256::digest(uri.as_bytes())))
}

pub fn load_override(directory: &Path, uri: &str) -> Option<Lyrics> {
    let mut lyrics: Lyrics =
        serde_json::from_str(&bounded_text(&override_path(directory, uri)).ok()?).ok()?;
    validate(&lyrics).ok()?;
    lyrics.imported = true;
    Some(lyrics)
}

pub fn import(directory: &Path, uri: &str, path: &Path) -> Result<Lyrics> {
    ensure!(
        uri.starts_with("spotify:track:") || uri.starts_with("spotify:local:"),
        "Choose a song for these lyrics"
    );
    let text = bounded_text(path)?;
    let text = text.trim_start_matches('\u{feff}');
    let mut lyrics = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ttml"))
        || text.trim_start().starts_with("<?xml")
        || text.trim_start().starts_with("<tt")
    {
        parse_ttml(text)?
    } else {
        super::from_text(text).context("No lyrics in that file")?
    };
    validate(&lyrics)?;
    let bytes = serde_json::to_vec(&lyrics)?;
    ensure!(bytes.len() <= MAX_BYTES, "Normalized lyrics exceed 2 MiB");
    std::fs::create_dir_all(directory)?;
    fastframe_file::write(&override_path(directory, uri), &bytes)?;
    lyrics.imported = true;
    Ok(lyrics)
}

pub fn clear_override(directory: &Path, uri: &str) -> Result<()> {
    match std::fs::remove_file(override_path(directory, uri)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn validate(lyrics: &Lyrics) -> Result<()> {
    ensure!(
        !lyrics.lines.is_empty() && lyrics.lines.len() <= MAX_LINES,
        "Invalid lyrics line count"
    );
    ensure!(
        lyrics.lines.iter().any(|line| !line.text.trim().is_empty()),
        "No lyrics in that file"
    );
    let mut previous_line = 0;
    let mut count = 0;
    for line in &lyrics.lines {
        if lyrics.synced {
            let start = line.at_ms.context("Missing line timing")?;
            ensure!(
                start >= previous_line && start <= MAX_TIME,
                "Invalid line timing order"
            );
            ensure!(
                line.end_ms.is_none_or(|end| end > start && end <= MAX_TIME),
                "Invalid line end"
            );
            previous_line = start;
        }
        let mut previous_byte = 0;
        let mut previous_end = line.at_ms.unwrap_or(0);
        for word in &line.words {
            count += 1;
            ensure!(count <= MAX_WORDS, "Too many timed words");
            ensure!(
                lyrics.synced
                    && word.start_ms >= previous_end
                    && word.end_ms > word.start_ms
                    && word.end_ms <= MAX_TIME,
                "Invalid word timing"
            );
            ensure!(
                line.end_ms.is_none_or(|end| word.end_ms <= end),
                "Word extends beyond its line"
            );
            ensure!(
                word.byte_start >= previous_byte
                    && word.byte_end > word.byte_start
                    && line
                        .text
                        .get(word.byte_start..word.byte_end)
                        .is_some_and(|text| !text.trim().is_empty()),
                "Invalid timed text range"
            );
            previous_byte = word.byte_end;
            previous_end = word.end_ms;
        }
    }
    Ok(())
}

/// Checked decimal clocks and offsets. Frames, ticks and nonfinite values
/// are deliberately unsupported rather than being silently mistimed.
fn time_ms(text: &str) -> Option<u32> {
    fn decimal(text: &str, unit: u64) -> Option<u32> {
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        if whole.is_empty()
            || !whole.bytes().all(|c| c.is_ascii_digit())
            || !fraction.bytes().all(|c| c.is_ascii_digit())
        {
            return None;
        }
        let mut value = whole.parse::<u64>().ok()?.checked_mul(unit)?;
        if !fraction.is_empty() {
            let n = fraction.len().min(9);
            value = value.checked_add(
                fraction[..n].parse::<u64>().ok()?.checked_mul(unit)? / 10u64.pow(n as u32),
            )?;
        }
        u32::try_from(value).ok().filter(|value| *value <= MAX_TIME)
    }
    let text = text.trim();
    for (suffix, unit) in [("ms", 1), ("h", 3_600_000), ("m", 60_000), ("s", 1_000)] {
        if let Some(number) = text.strip_suffix(suffix) {
            return decimal(number, unit);
        }
    }
    let parts: Vec<_> = text.split(':').collect();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (0, minutes.parse::<u32>().ok()?, decimal(seconds, 1_000)?),
        [hours, minutes, seconds] => {
            let minutes = minutes.parse::<u32>().ok()?;
            if minutes >= 60 {
                return None;
            }
            (
                hours.parse::<u32>().ok()?,
                minutes,
                decimal(seconds, 1_000)?,
            )
        }
        _ => return None,
    };
    if seconds >= 60_000 {
        return None;
    }
    hours
        .checked_mul(3_600_000)?
        .checked_add(minutes.checked_mul(60_000)?)?
        .checked_add(seconds)
        .filter(|value| *value <= MAX_TIME)
}

#[derive(Clone, Copy)]
struct Interval {
    start: u32,
    end: Option<u32>,
}

fn interval(node: Node<'_, '_>, parent: Interval, absolute: bool) -> Result<Interval> {
    let parse = |name| -> Result<Option<u32>> {
        node.attribute(name)
            .map(|value| time_ms(value).context("Unsupported TTML time expression"))
            .transpose()
    };
    let origin = if absolute { 0 } else { parent.start };
    let start = match parse("begin")? {
        Some(value) => origin.checked_add(value).context("TTML time overflow")?,
        None => parent.start,
    };
    let explicit_end = parse("end")?
        .map(|value| origin.checked_add(value).context("TTML time overflow"))
        .transpose()?;
    let duration_end = parse("dur")?
        .map(|value| start.checked_add(value).context("TTML duration overflow"))
        .transpose()?;
    let end = [explicit_end, duration_end, parent.end]
        .into_iter()
        .flatten()
        .min();
    ensure!(
        start <= MAX_TIME && end.is_none_or(|end| end > start && end <= MAX_TIME),
        "Invalid TTML interval"
    );
    Ok(Interval { start, end })
}

fn skipped(node: Node<'_, '_>) -> bool {
    node.attributes().any(|attr| {
        attr.name() == "role"
            && matches!(
                attr.value(),
                "x-bg" | "x-translation" | "x-roman" | "x-transliteration"
            )
    })
}

fn append_text(target: &mut String, text: &str) {
    for character in text.chars() {
        if character.is_whitespace() {
            if !target.is_empty() && !target.ends_with(' ') {
                target.push(' ');
            }
        } else {
            target.push(character);
        }
    }
}

fn spans(
    node: Node<'_, '_>,
    parent: Interval,
    absolute: bool,
    line: &mut Line,
    depth: usize,
) -> Result<()> {
    ensure!(depth < 128, "TTML nesting is too deep");
    if skipped(node) {
        return Ok(());
    }
    if node.is_text() {
        append_text(&mut line.text, node.text().unwrap_or_default());
        return Ok(());
    }
    if !node.is_element() {
        return Ok(());
    }
    let timing = interval(node, parent, absolute)?;
    let first = line.text.len();
    let word_count = line.words.len();
    for child in node.children() {
        spans(child, timing, absolute, line, depth + 1)?;
    }
    if node.tag_name().name() == "span"
        && line.words.len() == word_count
        && ["begin", "end", "dur"]
            .iter()
            .any(|name| node.has_attribute(*name))
    {
        let text = &line.text[first..];
        let start = first + text.len() - text.trim_start().len();
        let end = first + text.trim_end().len();
        if end > start {
            line.words.push(Word {
                start_ms: timing.start,
                end_ms: timing.end.context("Timed words need an end or duration")?,
                byte_start: start,
                byte_end: end,
            });
        }
    }
    Ok(())
}

pub fn parse_ttml(text: &str) -> Result<Lyrics> {
    ensure!(text.len() <= MAX_BYTES, "TTML exceeds 2 MiB");
    let document = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 80_000,
        },
    )
    .context("Invalid TTML document")?;
    let root = document.root_element();
    ensure!(root.tag_name().name() == "tt", "Expected a TTML document");
    let profile = root
        .attributes()
        .find(|attribute| attribute.name() == "timing")
        .map(|attribute| attribute.value());
    ensure!(
        matches!(profile, None | Some("None" | "Line" | "Word")),
        "Unsupported TTML lyric timing profile"
    );
    let absolute = profile.is_some();
    let mut lines = Vec::new();
    for paragraph in root
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "p")
    {
        if paragraph.ancestors().any(skipped)
            || !paragraph
                .ancestors()
                .any(|node| node.is_element() && node.tag_name().name() == "body")
        {
            continue;
        }
        ensure!(lines.len() < MAX_LINES, "Too many lyric lines");
        let mut parents: Vec<_> = paragraph
            .ancestors()
            .filter(|node| node.is_element())
            .skip(1)
            .collect();
        ensure!(parents.len() < 128, "TTML nesting is too deep");
        parents.reverse();
        let mut timing = Interval {
            start: 0,
            end: None,
        };
        for parent in parents {
            timing = interval(parent, timing, absolute)?;
        }
        let own = interval(paragraph, timing, absolute)?;
        let has_time = paragraph
            .ancestors()
            .chain(paragraph.descendants())
            .filter(|node| node.is_element() && !skipped(*node))
            .any(|node| {
                ["begin", "end", "dur"]
                    .iter()
                    .any(|name| node.has_attribute(*name))
            });
        let mut line = Line {
            at_ms: has_time.then_some(own.start),
            end_ms: own.end,
            ..Default::default()
        };
        spans(paragraph, timing, absolute, &mut line, 0)?;
        if profile == Some("None") {
            line.at_ms = None;
            line.end_ms = None;
            line.words.clear();
        }
        line.text.truncate(line.text.trim_end().len());
        if !line.text.is_empty() {
            if !paragraph.has_attribute("begin")
                && let Some(first) = line.words.first()
            {
                line.at_ms = Some(first.start_ms);
            }
            if line.end_ms.is_none() {
                line.end_ms = line.words.last().map(|word| word.end_ms);
            }
            lines.push(line);
        }
    }
    lines.sort_by_key(|line| line.at_ms);
    let lyrics = Lyrics {
        synced: !lines.is_empty() && lines.iter().all(|line| line.at_ms.is_some()),
        lines,
        ..Default::default()
    };
    validate(&lyrics)?;
    Ok(lyrics)
}

pub(super) fn lrc_line(body: &str, at_ms: u32, base_ms: u32) -> Line {
    let mut text = String::new();
    let mut words = Vec::new();
    let mut rest = body;
    let mut stamp = None;
    let mut start_byte = 0;
    while let Some(open) = rest.find('<') {
        let Some(close) = rest[open + 1..].find('>').map(|n| open + n + 1) else {
            break;
        };
        let Some(time) = time_ms(&rest[open + 1..close]) else {
            break;
        };
        let shifted = i64::from(time) + i64::from(at_ms) - i64::from(base_ms);
        let Ok(time) = u32::try_from(shifted) else {
            break;
        };
        text.push_str(&rest[..open]);
        if let Some(previous) = stamp
            && text.len() > start_byte
            && !text[start_byte..].trim().is_empty()
        {
            words.push(Word {
                start_ms: previous,
                end_ms: time,
                byte_start: start_byte,
                byte_end: text.len(),
            });
        }
        stamp = Some(time);
        start_byte = text.len();
        rest = &rest[close + 1..];
    }
    text.push_str(rest);
    if let Some(previous) = stamp
        && text.len() > start_byte
        && !text[start_byte..].trim().is_empty()
    {
        words.push(Word {
            start_ms: previous,
            end_ms: 0,
            byte_start: start_byte,
            byte_end: text.len(),
        });
    }
    Line {
        at_ms: Some(at_ms),
        text,
        words,
        ..Default::default()
    }
}

pub(super) fn finish_lrc(lines: &mut [Line]) {
    for index in 0..lines.len() {
        let next = lines.get(index + 1).and_then(|line| line.at_ms);
        let line = &mut lines[index];
        if let Some(last) = line.words.last_mut()
            && last.end_ms == 0
        {
            if let Some(next) = next {
                last.end_ms = next;
            } else {
                line.words.clear();
            }
        }
        if !line.words.is_empty() {
            line.end_ms = line.words.last().map(|word| word.end_ms);
            let candidate = Lyrics {
                lines: vec![line.clone()],
                synced: true,
                ..Default::default()
            };
            if validate(&candidate).is_err() {
                line.words.clear();
                line.end_ms = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const APPLE: &str = r#"<tt xmlns:i="urn:i" i:timing="Word"><body><div begin="10s"><p begin="12s" end="15s"><span begin="12s" end="13s">Caf&#233;</span> <span begin="13s" end="15s">&#19990;&#30028; &amp; sky</span><span xmlns:m="urn:m" m:role="x-bg" begin="12s" end="15s">echo</span></p></div></body></tt>"#;

    #[test]
    fn apple_absolute_words_preserve_unicode_ranges_and_skip_background() {
        let lyrics = parse_ttml(APPLE).unwrap();
        let line = &lyrics.lines[0];
        assert_eq!((line.at_ms, line.end_ms), (Some(12_000), Some(15_000)));
        assert_eq!(line.text, "Caf\u{e9} \u{4e16}\u{754c} & sky");
        assert_eq!(
            &line.text[line.words[0].byte_start..line.words[0].byte_end],
            "Caf\u{e9}"
        );
        assert_eq!(
            (line.words[1].start_ms, line.words[1].end_ms),
            (13_000, 15_000)
        );
        assert_eq!(lyrics.active_line(11_999), None);
        assert_eq!(lyrics.active_line(12_000), Some(0));
        assert_eq!(lyrics.active_line(15_000), None);
    }

    #[test]
    fn standard_ttml_composes_parent_relative_intervals() {
        let lyrics = parse_ttml(r#"<tt><body begin="10s"><div begin="2s"><p begin="3s" dur="4s"><span begin="0s" dur="1s">First</span> <span begin="1s" dur="2s">second</span></p></div></body></tt>"#).unwrap();
        let line = &lyrics.lines[0];
        assert_eq!((line.at_ms, line.end_ms), (Some(15_000), Some(19_000)));
        assert_eq!(
            (line.words[1].start_ms, line.words[1].end_ms),
            (16_000, 18_000)
        );
    }

    #[test]
    fn static_ttml_and_untimed_text_stay_untimed() {
        let lyrics = parse_ttml(r#"<tt xmlns:i="urn:i" i:timing="None"><body><p begin="1s" end="2s"><span begin="1s" end="2s">Hello</span></p></body></tt>"#).unwrap();
        assert!(!lyrics.synced);
        assert_eq!(lyrics.lines[0].at_ms, None);
        assert!(lyrics.lines[0].words.is_empty());
        let plain = super::super::from_text("Love <letters> stay").unwrap();
        assert_eq!(plain.lines[0].text, "Love <letters> stay");
    }

    #[test]
    fn unsafe_or_unsupported_ttml_is_rejected() {
        for clock in [
            "NaNs",
            "-1s",
            "00:61.0",
            "00:60:01",
            "999999999999999999999h",
            "25h",
            "30f",
            "1t",
        ] {
            assert!(time_ms(clock).is_none(), "{clock}");
        }
        for xml in [
            r#"<!DOCTYPE tt [<!ENTITY x "expanded">]><tt><body><p>&x;</p></body></tt>"#,
            r#"<tt><body><p begin="0s" end="4s"><span begin="0s" end="2s">one</span><span begin="1s" end="3s">two</span></p></body></tt>"#,
            "<tt><body><p>  </p></body></tt>",
        ] {
            assert!(parse_ttml(xml).is_err(), "{xml}");
        }
        let deep = format!(
            "<tt><body>{}<p>words</p>{}</body></tt>",
            "<div>".repeat(130),
            "</div>".repeat(130)
        );
        assert!(parse_ttml(&deep).is_err());
        assert!(parse_ttml(&" ".repeat(MAX_BYTES + 1)).is_err());
    }

    #[test]
    fn enhanced_lrc_uses_real_boundaries_and_repeated_line_offsets() {
        let lyrics =
            super::super::from_text("[00:10][00:20]<00:10>Cafe <00:11>sky<00:12>\n[00:30]next")
                .unwrap();
        assert_eq!(lyrics.lines[0].text, "Cafe sky");
        assert_eq!(lyrics.lines[0].words[1].end_ms, 12_000);
        assert_eq!(lyrics.lines[1].words[0].start_ms, 20_000);
        assert_eq!(lyrics.lines[1].words[1].end_ms, 22_000);
        let inferred =
            super::super::from_text("[00:01]<00:01>one <00:02>two\n[00:04]next").unwrap();
        assert_eq!(inferred.lines[0].words[1].end_ms, 4_000);
        let final_line = super::super::from_text("[00:01]<00:01>no closing marker").unwrap();
        assert!(final_line.lines[0].words.is_empty());
        let backwards = super::super::from_text("[00:01]<00:02>one<00:01>two<00:03>").unwrap();
        assert!(backwards.lines[0].words.is_empty());
        assert_eq!(backwards.lines[0].text, "onetwo");
    }

    #[test]
    fn legacy_cached_lyrics_remain_readable() {
        let lyrics: Lyrics = serde_json::from_str(
            r#"{"lines":[{"at_ms":1000,"text":"old cache"}],"synced":true,"instrumental":false}"#,
        )
        .unwrap();
        assert!(lyrics.lines[0].words.is_empty());
        assert_eq!(lyrics.lines[0].end_ms, None);
        assert!(!lyrics.imported);
    }

    #[test]
    fn validation_rejects_invalid_utf8_ranges_and_word_order() {
        let mut lyrics = parse_ttml(APPLE).unwrap();
        lyrics.lines[0].words[0].byte_end = 4;
        assert!(validate(&lyrics).is_err());
        let mut lyrics = parse_ttml(APPLE).unwrap();
        lyrics.lines[0].words[1].start_ms = 12_500;
        assert!(validate(&lyrics).is_err());
    }

    #[test]
    fn durable_import_is_track_specific_and_never_changes_the_original() {
        let root = std::env::temp_dir().join(format!(
            "spotifast-timed-import-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let original = root.join("source.ttml");
        std::fs::write(&original, APPLE).unwrap();
        let directory = root.join("lyrics");
        let uri = "spotify:track:example";
        let imported = import(&directory, uri, &original).unwrap();
        assert!(imported.imported);
        assert_eq!(load_override(&directory, uri), Some(imported));
        assert!(load_override(&directory, "spotify:track:other").is_none());
        let invalid = root.join("broken.ttml");
        std::fs::write(&invalid, "<tt>broken").unwrap();
        assert!(import(&directory, uri, &invalid).is_err());
        assert!(load_override(&directory, uri).is_some());
        clear_override(&directory, uri).unwrap();
        assert!(load_override(&directory, uri).is_none());
        assert_eq!(std::fs::read_to_string(&original).unwrap(), APPLE);
        clear_override(&directory, uri).unwrap();
        assert!(import(&directory, "../../escape", &original).is_err());
    }
}
