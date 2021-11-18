use anyhow::Context;

use crate::monster::MonsterKindMask;
use crate::ResistMask;

pub(crate) fn trim_ascii(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_ascii_whitespace())
}

pub(crate) fn trim_start_ascii(s: &str) -> &str {
    s.trim_start_matches(|c: char| c.is_ascii_whitespace())
}

pub(crate) fn parse_resist_mask(s: impl AsRef<str>) -> anyhow::Result<ResistMask> {
    let s = s.as_ref();

    let mut bits = 0;

    for c in s.chars() {
        let element = c
            .to_digit(16)
            .with_context(|| format!("invalid element char: {}", c))?;

        bits |= 1 << element;
    }

    let mask = ResistMask::from_bits(bits)
        .with_context(|| format!("unknown resist mask bit: {:#b}", bits))?;

    Ok(mask)
}

pub(crate) fn parse_monster_kind_mask(s: impl AsRef<str>) -> anyhow::Result<MonsterKindMask> {
    let s = s.as_ref();

    let mut bits = 0;

    for c in s.chars() {
        let kind = c
            .to_digit(16)
            .with_context(|| format!("invalid monster kind char: {}", c))?;

        bits |= 1 << kind;
    }

    let mask = MonsterKindMask::from_bits(bits)
        .with_context(|| format!("unknown monster kind mask bit: {:#b}", bits))?;

    Ok(mask)
}
