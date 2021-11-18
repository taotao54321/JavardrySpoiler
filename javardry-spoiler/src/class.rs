use anyhow::{anyhow, bail, ensure, Context};

use crate::kvs::{Kvs, KvsExt};
use crate::monster::MonsterKindMask;
use crate::util;
use crate::DebuffMask;

#[derive(Debug)]
pub struct Class {
    pub id: u32,
    pub name: String,
    pub name_abbr: String,
    pub sex_mask: u8,
    pub alignment_mask: u8,
    pub stats: Vec<u32>,
    pub ac_expr: String,
    pub hit_expr: String,
    pub attack_count_expr: String,
    pub barehand_damage_expr: [String; 3],
    pub attack_debuff_mask: DebuffMask,
    pub thief_skill: i32,
    pub can_identify: bool,
    pub xl_for_dispell: Option<u32>,
    pub dispell_mask: MonsterKindMask,
    pub hp_expr: String,
    pub xp_expr: String,
    pub description: String,
    pub inven_bonus: i32,
    pub cond_to_appear: String,
    // TODO: 呪文関連
    // TODO: 汎用修正値
}

pub(crate) fn classes_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<Class>> {
    let mut classes = Vec::<Class>::new();

    for (i, text) in kvs.iter_seq("Class").enumerate() {
        let id = u32::try_from(i).expect("class id should be u32");
        let class = parse(id, text).map_err(|e| anyhow!("class {}: {}", id, e))?;
        classes.push(class);
    }

    Ok(classes)
}

fn parse(id: u32, text: impl AsRef<str>) -> anyhow::Result<Class> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<>").collect();
    ensure!(fields.len() == 21, "class text must have 21 fields");

    let name = fields[0].to_owned();
    let name_abbr = fields[1].to_owned();
    let sex_mask = parse_sex_mask(fields[2])?;
    let alignment_mask = parse_alignment_mask(fields[3])?;
    let stats: Vec<_> = fields[4]
        .split(',')
        .map(str::parse::<u32>)
        .collect::<Result<_, _>>()?;
    let ac_expr = fields[5].to_owned();
    let hit_expr = fields[6].to_owned();
    let attack_count_expr = fields[7].to_owned();
    let barehand_damage_expr = parse_barehand_damage_expr(fields[8])?;
    let attack_debuff_mask = parse_attack_debuff_mask(fields[9])?;
    let thief_skill: i32 = fields[10].parse()?;
    let can_identify: bool = fields[11].parse()?;
    let xl_for_dispell = {
        let xl: u32 = fields[12].parse()?;
        (xl != 0).then(|| xl)
    };
    let dispell_mask = util::parse_monster_kind_mask(fields[13])?;
    let hp_expr = fields[15].to_owned();
    let xp_expr = fields[16].to_owned();
    let description = fields[17].to_owned();
    let inven_bonus: i32 = fields[18].parse()?;
    let cond_to_appear = fields[20].to_owned();

    Ok(Class {
        id,
        name,
        name_abbr,
        sex_mask,
        alignment_mask,
        stats,
        ac_expr,
        hit_expr,
        attack_count_expr,
        barehand_damage_expr,
        attack_debuff_mask,
        thief_skill,
        can_identify,
        xl_for_dispell,
        dispell_mask,
        hp_expr,
        xp_expr,
        description,
        inven_bonus,
        cond_to_appear,
    })
}

fn parse_sex_mask(s: &str) -> anyhow::Result<u8> {
    let mut mask = 0;

    for c in s.chars() {
        let sex = c
            .to_digit(10)
            .with_context(|| format!("invalid sex char: {}", c))?;
        ensure!(sex < 2, "invalid sex: {}");

        mask |= 1 << sex;
    }

    Ok(mask)
}

fn parse_alignment_mask(s: &str) -> anyhow::Result<u8> {
    let mut mask = 0;

    for c in s.chars() {
        let alignment = c
            .to_digit(10)
            .with_context(|| format!("invalid alignment char: {}", c))?;
        ensure!(alignment < 3, "invalid alignment: {}");

        mask |= 1 << alignment;
    }

    Ok(mask)
}

fn parse_barehand_damage_expr(s: &str) -> anyhow::Result<[String; 3]> {
    let fields: Vec<_> = s.split(',').collect();
    ensure!(fields.len() == 3, "barehand damage expr must have 3 fields");

    Ok(fields
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .try_into()
        .expect("fields.len() should be 3"))
}

fn parse_attack_debuff_mask(s: &str) -> anyhow::Result<DebuffMask> {
    let value: u8 = s.parse()?;

    let mask = match value {
        0 => DebuffMask::empty(),
        1 => DebuffMask::KNOCKOUT,
        2 => DebuffMask::CRITICAL,
        _ => bail!("invalid class attack debuff value: {}", value),
    };

    Ok(mask)
}
