//! Rewriting of a message another validator wrote, so that it no longer
//! quotes the flagged values of the segment it is about.
//!
//! A value is bytes and the message is text, so a value is looked for read
//! as UTF-8 and, when that differs, read as Latin-1 (one character per
//! byte), the way a validator that decodes its input as Latin-1 quotes it.
//! Its length is counted once, in characters of its UTF-8 reading, and
//! holds for both readings. A value of 4 characters or more is replaced
//! wherever it occurs. A shorter one is replaced only where it stands apart:
//! the character on each side is the edge of the message, a character that
//! is not an ASCII letter or digit (an accented letter or `-` counts as
//! apart), or part of another replaced value. Short values that overlap or
//! touch form a group, and a group that stands apart as a whole is replaced
//! whole, so values glued only to each other (`AB` and `CD` in `(ABCD)`) are
//! replaced. A short value glued to a letter or digit that no replaced value
//! holds stays as written, so the words and numbers of the message keep
//! their letters, and so does a short value inside such a word: with `XAB`
//! and `ABYZ` flagged, `QXABYZ` becomes `QX` and the masked form of `ABYZ`.
//! Two short values that together spell a word of the message replace it.
//!
//! The replaced values are written as their masked forms in the order they
//! start, a value held in a longer one giving way to it, and each place of
//! the message is written once, so a masked form is never searched again.

/// Values shorter than this many characters are replaced only where they
/// stand apart from the words and numbers around them.
const SHORT: usize = 4;

/// `message` with the originals of `quoted` it holds replaced by their
/// masked forms, as the module notes say.
pub(super) fn unquoted(message: &str, quoted: &[(Vec<u8>, Vec<u8>)]) -> String {
    let mut patterns: Vec<(String, String, bool)> = Vec::new();
    for (original, masked) in quoted {
        if original.is_empty() {
            continue;
        }
        let masked = String::from_utf8_lossy(masked).into_owned();
        let utf8 = String::from_utf8_lossy(original).into_owned();
        let short = utf8.chars().count() < SHORT;
        let latin1: String = original.iter().copied().map(char::from).collect();
        if latin1 != utf8 {
            patterns.push((latin1, masked.clone(), short));
        }
        patterns.push((utf8, masked, short));
    }
    let mut matches: Vec<Match<'_>> = Vec::new();
    let mut pending: Vec<Match<'_>> = Vec::new();
    for (start, _) in message.char_indices() {
        let Some(rest) = message.get(start..) else {
            continue;
        };
        for (from, to, short) in &patterns {
            if rest.starts_with(from.as_str()) {
                let found = Match {
                    start,
                    end: start + from.len(),
                    to,
                };
                if *short {
                    pending.push(found);
                } else {
                    matches.push(found);
                }
            }
        }
    }
    pending.sort_by_key(|found| (found.start, found.end));
    loop {
        let take = accepted(message, &pending, &matches);
        if !take.contains(&true) {
            break;
        }
        let mut rest = Vec::with_capacity(pending.len());
        for (found, take) in pending.into_iter().zip(take) {
            if take {
                matches.push(found);
            } else {
                rest.push(found);
            }
        }
        pending = rest;
    }
    written(message, matches)
}

/// One place of a message where an original starts, as byte offsets, with
/// its masked form.
struct Match<'p> {
    start: usize,
    end: usize,
    to: &'p str,
}

/// Which of `pending`, sorted by start, stand apart given the accepted
/// `matches`: one that does on its own, and every one of a group of
/// overlapping or touching matches that does as a whole.
fn accepted(message: &str, pending: &[Match<'_>], matches: &[Match<'_>]) -> Vec<bool> {
    let mut take: Vec<bool> = pending
        .iter()
        .map(|found| stands_apart(message, found.start, found.end, matches))
        .collect();
    let mut groups: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (index, found) in pending.iter().enumerate() {
        match groups.last_mut() {
            Some((_, last, _, end)) if found.start <= *end => {
                *last = index + 1;
                *end = (*end).max(found.end);
            }
            _ => groups.push((index, index + 1, found.start, found.end)),
        }
    }
    for (first, last, start, end) in groups {
        if stands_apart(message, start, end, matches) {
            for one in take.iter_mut().take(last).skip(first) {
                *one = true;
            }
        }
    }
    take
}

/// Whether the character on each side of `start..end` is the edge of
/// `message`, not an ASCII letter or digit, or held in one of `matches`.
fn stands_apart(message: &str, start: usize, end: usize, matches: &[Match<'_>]) -> bool {
    let held = |at: usize| matches.iter().any(|m| m.start <= at && at < m.end);
    let before = message.get(..start).and_then(|head| {
        let c = head.chars().next_back()?;
        Some((start - c.len_utf8(), c))
    });
    let after = message
        .get(end..)
        .and_then(|tail| tail.chars().next())
        .map(|c| (end, c));
    [before, after].into_iter().all(|side| match side {
        None => true,
        Some((at, c)) => !c.is_ascii_alphanumeric() || held(at),
    })
}

/// `message` with each of `matches` written as its masked form, in the
/// order they start; a match held in an earlier, longer one is dropped.
fn written(message: &str, mut matches: Vec<Match<'_>>) -> String {
    matches.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
    let mut out = String::with_capacity(message.len());
    let mut written = 0;
    for found in matches {
        if found.end <= written {
            continue;
        }
        if found.start > written {
            out.push_str(message.get(written..found.start).unwrap_or_default());
        }
        out.push_str(found.to);
        written = found.end;
    }
    out.push_str(message.get(written..).unwrap_or_default());
    out
}
