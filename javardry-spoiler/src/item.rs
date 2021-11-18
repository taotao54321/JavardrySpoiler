use anyhow::{anyhow, bail, ensure, Context};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::kvs::{Kvs, KvsExt};
use crate::monster::MonsterKindMask;
use crate::util;
use crate::{DebuffMask, ResistMask};

#[derive(Debug)]
pub struct Item {
    pub id: u32,
    pub name_ident: String,
    pub name_unident: String,
    pub kind: ItemKind,
    pub price: u64,
    pub stock: i32,
    pub equip_class_mask: u64,
    pub equip_race_mask: u64,
    pub curse_alignment_mask: u8,
    pub curse_sex_mask: u8,
    pub ac: i32,
    pub ac_curse: i32,
    pub damage_expr: [String; 3],
    pub hit_modifier: i32,
    pub attack_count_modifier: i32,
    pub attack_debuff_mask: DebuffMask,
    pub healing: i32,
    pub resist_mask: ResistMask,
    pub spell_cancel: i32,
    pub slay_mask: MonsterKindMask,
    pub protect_mask: MonsterKindMask,
    pub use_str: String, // 使用効果。仕様が理解しきれてないのでとりあえず生文字列
    pub sp_str: String,  // SP。仕様が理解(ry
    pub break_prob_expr: String,
    pub broken_item_id: Option<u32>,
    pub description: String,
    pub ident_difficulty: u32,
    pub attack_target_count: u32,
    pub usable_only_if_equipable: bool,
    pub effect_only_if_equiped: bool,
    pub disable_class_attack_debuff_if_equiped: bool,
    pub disable_class_ac_if_equiped: bool,
    pub stats_bonus: Vec<i32>,
    pub halve_attack_count_if_subweapon: bool,
    pub poison_damage: u32,
    pub effect_only_if_equipable: bool,
    pub hide_in_catalog: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ItemKind {
    Weapon = 0,
    Armor = 1,
    Shield = 2,
    Helmet = 3,
    Gloves = 4,
    Boots = 5,
    Tool = 6,
}

pub(crate) fn items_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<Item>> {
    let mut items = Vec::<Item>::new();

    for (i, text) in kvs.iter_seq("Item").enumerate() {
        let id = u32::try_from(i).expect("item id should be u32");
        let item = parse(id, text).map_err(|e| anyhow!("item {}: {}", id, e))?;
        items.push(item);
    }

    Ok(items)
}

fn parse(id: u32, text: impl AsRef<str>) -> anyhow::Result<Item> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<>").collect();
    ensure!(fields.len() == 39, "item text must have 39 fields");

    let name_ident = fields[0].to_owned();
    let name_unident = fields[1].to_owned();
    let kind: ItemKind = fields[2].parse::<u8>()?.try_into()?;
    let price: u64 = fields[3].parse()?;
    let stock: i32 = fields[4].parse()?;
    let (equip_class_mask, equip_race_mask) = parse_equip_masks(fields[5])?;
    let (curse_alignment_mask, curse_sex_mask) = parse_curse_masks(fields[6])?;
    let ac: i32 = fields[8].parse()?;
    let ac_curse: i32 = fields[9].parse()?;
    let damage_expr = parse_damage_expr(fields[10])?;

    // TODO: fields[15]: range

    let hit_modifier: i32 = fields[12].parse()?;
    let attack_count_modifier: i32 = fields[13].parse()?;
    let attack_debuff_mask = parse_attack_debuff_mask(fields[14])?;
    let healing: i32 = fields[18].parse()?;
    let resist_mask = util::parse_resist_mask(fields[22])?;
    let spell_cancel: i32 = fields[19].parse()?;
    let slay_mask = util::parse_monster_kind_mask(fields[16])?;
    let protect_mask = util::parse_monster_kind_mask(fields[17])?;
    let use_str = fields[24].to_owned();
    let sp_str = fields[25].to_owned();
    let break_prob_expr = fields[20].to_owned();
    let broken_item_id = parse_broken_item_id(fields[21])?;
    let description = fields[23].to_owned();
    let ident_difficulty: u32 = fields[7].parse()?;

    // TODO: fields[11]: attack kind

    let attack_target_count: u32 = fields[26].parse()?;

    // TODO: fields[27]: weapon kind

    let usable_only_if_equipable: bool = fields[28].parse()?;
    let effect_only_if_equiped: bool = fields[29].parse()?;
    let disable_class_attack_debuff_if_equiped: bool = fields[30].parse()?;
    let disable_class_ac_if_equiped: bool = fields[31].parse()?;
    let stats_bonus = parse_stats_bonus(fields[32])?;
    let halve_attack_count_if_subweapon: bool = fields[33].parse()?;
    let poison_damage: u32 = fields[34].parse()?;
    let effect_only_if_equipable: bool = fields[35].parse()?;
    let hide_in_catalog: bool = fields[36].parse()?;

    // TODO: fields[37]: 戦闘メッセージ
    // TODO: fields[38]: 確定状態

    Ok(Item {
        id,
        name_ident,
        name_unident,
        kind,
        price,
        stock,
        equip_class_mask,
        equip_race_mask,
        curse_alignment_mask,
        curse_sex_mask,
        ac,
        ac_curse,
        damage_expr,
        hit_modifier,
        attack_count_modifier,
        attack_debuff_mask,
        healing,
        resist_mask,
        spell_cancel,
        slay_mask,
        protect_mask,
        use_str,
        sp_str,
        break_prob_expr,
        broken_item_id,
        description,
        ident_difficulty,
        attack_target_count,
        usable_only_if_equipable,
        effect_only_if_equiped,
        disable_class_attack_debuff_if_equiped,
        disable_class_ac_if_equiped,
        stats_bonus,
        halve_attack_count_if_subweapon,
        poison_damage,
        effect_only_if_equipable,
        hide_in_catalog,
    })
}

fn parse_equip_masks(s: &str) -> anyhow::Result<(u64, u64)> {
    if s.is_empty() {
        return Ok((0, 0));
    }

    let fields: Vec<_> = s.split(',').collect();
    ensure!(fields.len() == 2, "equip mask string must have 2 fields");

    let equip_class_mask = parse_equip_class_mask(fields[0])?;
    let equip_race_mask = parse_equip_race_mask(fields[1])?;

    Ok((equip_class_mask, equip_race_mask))
}

fn parse_equip_class_mask(s: &str) -> anyhow::Result<u64> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\Aclass\[([0-9]+)\]\z").expect("regex should be valid"));

    if s == "-" {
        return Ok(0);
    }

    let mut mask = 0;

    for field in s.split("<+>") {
        let caps = RE
            .captures(field)
            .with_context(|| format!("invalid class string: {}", field))?;
        let class: u32 = caps
            .get(1)
            .expect("capture group 1 should exist")
            .as_str()
            .parse()?;
        ensure!(class < 36, "invalid class: {}", class);

        mask |= 1 << class;
    }

    Ok(mask)
}

fn parse_equip_race_mask(s: &str) -> anyhow::Result<u64> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\Arace\[([0-9]+)\]\z").expect("regex should be valid"));

    if s == "-" {
        return Ok(0);
    }

    let mut mask = 0;

    for field in s.split("<+>") {
        let caps = RE
            .captures(field)
            .with_context(|| format!("invalid race string: {}", field))?;
        let race: u32 = caps
            .get(1)
            .expect("capture group 1 should exist")
            .as_str()
            .parse()?;
        ensure!(race < 36, "invalid race: {}", race);

        mask |= 1 << race;
    }

    Ok(mask)
}

fn parse_curse_masks(s: &str) -> anyhow::Result<(u8, u8)> {
    if s.is_empty() {
        return Ok((0, 0));
    }

    let fields: Vec<_> = s.split(',').collect();
    ensure!(fields.len() == 2, "curse mask string must have 2 fields");

    let curse_alignment_mask = parse_curse_alignment_mask(fields[0])?;
    let curse_sex_mask = parse_curse_sex_mask(fields[1])?;

    Ok((curse_alignment_mask, curse_sex_mask))
}

fn parse_curse_alignment_mask(s: &str) -> anyhow::Result<u8> {
    if s == "-" {
        return Ok(0);
    }

    let mut mask = 0;

    for c in s.chars() {
        let alignment = c
            .to_digit(10)
            .with_context(|| format!("invalid alignment char: {}", c))?;
        ensure!(alignment < 3, "invalid alignment: {}", alignment);

        mask |= 1 << alignment;
    }

    Ok(mask)
}

fn parse_curse_sex_mask(s: &str) -> anyhow::Result<u8> {
    if s == "-" {
        return Ok(0);
    }

    let mut mask = 0;

    for c in s.chars() {
        let sex = c
            .to_digit(10)
            .with_context(|| format!("invalid sex char: {}", c))?;
        ensure!(sex < 2, "invalid sex: {}", sex);

        mask |= 1 << sex;
    }

    Ok(mask)
}

fn parse_damage_expr(s: &str) -> anyhow::Result<[String; 3]> {
    let fields: Vec<_> = s.split(',').collect();
    ensure!(fields.len() == 3, "damage expr string must have 3 fields");

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
        3 => DebuffMask::SLEEP,
        4 => DebuffMask::PARALYSIS,
        5 => DebuffMask::PETRIFICATION,
        _ => bail!("invalid item attack debuff value: {}", value),
    };

    Ok(mask)
}

fn parse_broken_item_id(s: &str) -> anyhow::Result<Option<u32>> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\Aitem\[([0-9]+)\]\z").expect("regex should be valid"));

    if s == "-1" {
        return Ok(None);
    }

    let caps = RE
        .captures(s)
        .with_context(|| format!("invalid item string: {}", s))?;
    let item: u32 = caps
        .get(1)
        .expect("capture group 1 should exist")
        .as_str()
        .parse()?;

    Ok(Some(item))
}

fn parse_stats_bonus(s: &str) -> anyhow::Result<Vec<i32>> {
    Ok(s.split(',').map(str::parse).collect::<Result<_, _>>()?)
}
