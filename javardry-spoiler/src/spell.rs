use anyhow::{anyhow, ensure};

use crate::kvs::{Kvs, KvsExt};
use crate::util;

#[derive(Debug)]
pub struct SpellRealm {
    pub id: u32,
    pub name: String,
    pub level_count: u32,
    pub spells_of_levels: Vec<Vec<Spell>>,
    pub is_only_for_monster: bool,
}

#[derive(Debug)]
pub struct Spell {
    pub name: String,
    pub description: String,
    pub cost_mp: u32,
    pub ignore_silence: bool,
    pub extra_learn: bool, // レベルアップで習得しない
}

pub(crate) fn spell_realms_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<SpellRealm>> {
    let level_count: u32 = kvs.get_expect("SpellLvNum")?.parse()?;
    let last_realm_is_only_for_monster: bool = kvs.get_expect("ExclusiveUseOfMonsters")?.parse()?;

    let mut realms = Vec::<SpellRealm>::new();

    let mut it = kvs.iter_seq("SpellKind").enumerate().peekable();
    while let Some((i, text)) = it.next() {
        let is_last = it.peek().is_none();
        let id = u32::try_from(i).expect("spell realm id should be u32");
        let is_only_for_monster = last_realm_is_only_for_monster && is_last;
        let realm = parse(level_count, is_only_for_monster, id, text)
            .map_err(|e| anyhow!("spell realm {}: {}", id, e))?;
        realms.push(realm);
    }

    Ok(realms)
}

fn parse(
    level_count: u32,
    is_only_for_monster: bool,
    id: u32,
    text: impl AsRef<str>,
) -> anyhow::Result<SpellRealm> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<-->").collect();
    ensure!(
        fields.len() == usize::try_from(level_count).unwrap() + 1,
        "level count mismatch"
    );

    let name = fields[0].to_owned();
    let spells_of_levels: Vec<_> = fields[1..]
        .iter()
        .map(|&s| parse_spells_of_level(s))
        .collect::<Result<_, _>>()?;

    Ok(SpellRealm {
        id,
        name,
        level_count,
        spells_of_levels,
        is_only_for_monster,
    })
}

fn parse_spells_of_level(s: &str) -> anyhow::Result<Vec<Spell>> {
    let s = util::trim_ascii(s);
    if s.is_empty() {
        return Ok(vec![]);
    }

    let fields: Vec<_> = s.split("<++>").collect();

    let spells: Vec<_> = fields
        .into_iter()
        .map(parse_spell)
        .collect::<Result<_, _>>()?;

    Ok(spells)
}

fn parse_spell(s: &str) -> anyhow::Result<Spell> {
    let fields: Vec<_> = s.split("<>").collect();
    ensure!(fields.len() == 8, "spell text must have 8 fields");

    let name = fields[0].to_owned();
    let description = fields[2].to_owned();
    let cost_mp: u32 = fields[6].parse()?;
    let ignore_silence: bool = fields[7].parse()?;
    let extra_learn: bool = fields[5].parse()?;

    Ok(Spell {
        name,
        description,
        cost_mp,
        ignore_silence,
        extra_learn,
    })
}
