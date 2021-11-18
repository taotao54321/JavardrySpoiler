use anyhow::{anyhow, ensure, Context};
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::kvs::{Kvs, KvsExt};
use crate::{DebuffMask, ResistMask};

#[derive(Debug)]
pub struct Monster {
    pub id: u32,
    pub name_ident: String,
    pub name_unident: String,
    pub name_plural_ident: String,
    pub name_plural_unident: String,
    pub kind: MonsterKind,
    pub xl_expr: String,
    pub hp_expr: String,
    pub mp_expr: String,
    pub ac_expr: String,
    pub stats: Vec<u32>,
    pub damage_expr: String,
    pub attack_count_expr: String,
    pub attack_debuff_mask: DebuffMask,
    pub poison_damage: u32,
    pub drain_xl: u32,
    pub spell_levels: Vec<u32>,
    pub healing: i32,
    pub resist_mask: ResistMask,
    pub spell_cancel: i32,
    pub vuln_mask: ResistMask,
    pub can_flee: bool,
    pub can_call: bool,
    pub friendly_prob: u32,
    pub count_in_group_expr: String,
    pub follower: Option<MonsterFollower>,
    pub xp_expr: String,
    pub is_invincible: bool,
    pub attack_twice: bool,
    pub description: String,
    pub hide_in_catalog: bool,
    // TODO: 攻撃範囲
    // TODO: ブレス
    // TODO: 行動パターン
    // TODO: ドロップ関連
    // TODO: 攻撃種別
    // TODO: 画像
    // TODO: 戦闘メッセージ
    // TODO: 音楽
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MonsterKind {
    Fighter = 0,
    Mage = 1,
    Priest = 2,
    Thief = 3,
    Midget = 4, // 小人
    Giant = 5,
    Myth = 6,
    Dragon = 7,
    Animal = 8,
    Werecreature = 9,
    Undead = 10,
    Demon = 11,
    Insect = 12,
    Enchanted = 13, // 魔法生物
    Mystery = 14,   // 謎の生物
}

bitflags! {
    pub struct MonsterKindMask: u32 {
        const FIGHTER = 1 << (MonsterKind::Fighter as u8);
        const MAGE = 1 << (MonsterKind::Mage as u8);
        const PRIEST = 1 << (MonsterKind::Priest as u8);
        const THIEF = 1 << (MonsterKind::Thief as u8);
        const MIDGET = 1 << (MonsterKind::Midget as u8);
        const GIANT = 1 << (MonsterKind::Giant as u8);
        const MYTH = 1 << (MonsterKind::Myth as u8);
        const DRAGON = 1 << (MonsterKind::Dragon as u8);
        const ANIMAL = 1 << (MonsterKind::Animal as u8);
        const WERECREATURE = 1 << (MonsterKind::Werecreature as u8);
        const UNDEAD = 1 << (MonsterKind::Undead as u8);
        const DEMON = 1 << (MonsterKind::Demon as u8);
        const INSECT = 1 << (MonsterKind::Insect as u8);
        const ENCHANTED = 1 << (MonsterKind::Enchanted as u8);
        const MYSTERY = 1 << (MonsterKind::Mystery as u8);
    }
}

#[derive(Debug)]
pub struct MonsterFollower {
    pub id_expr: String,
    pub prob: u32,
}

pub(crate) fn monsters_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<Monster>> {
    let mut monsters = Vec::<Monster>::new();

    for (i, text) in kvs.iter_seq("Monster").enumerate() {
        let id = u32::try_from(i).expect("race id should be u32");
        let monster = parse(id, text).map_err(|e| anyhow!("monster {}: {}", id, e))?;
        monsters.push(monster);
    }

    Ok(monsters)
}

fn parse(id: u32, text: impl AsRef<str>) -> anyhow::Result<Monster> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<>").collect();
    ensure!(
        fields.len() >= 49,
        "monster text must have at least 49 fields"
    );

    let name_ident = fields[0].to_owned();
    let name_unident = fields[1].to_owned();
    let name_plural_ident = fields[2].to_owned();
    let name_plural_unident = fields[3].to_owned();
    let kind: MonsterKind = fields[4].parse::<u8>()?.try_into()?;
    let xl_expr = fields[5].to_owned();
    let hp_expr = fields[7].to_owned();
    let mp_expr = fields[8].to_owned();
    let ac_expr = fields[9].to_owned();
    let stats: Vec<u32> = fields[10]
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let damage_expr = fields[12].to_owned();
    let attack_count_expr = fields[13].to_owned();
    let attack_debuff_mask = parse_attack_debuff_mask(fields[19])?;
    let poison_damage: u32 = fields[14].parse()?;
    let drain_xl: u32 = fields[15].parse()?;
    let spell_levels: Vec<u32> = fields[18]
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let healing: i32 = fields[16].parse()?;
    let resist_mask = parse_resist_mask(fields[22])?;
    let spell_cancel: i32 = fields[17].parse()?;
    let vuln_mask = parse_resist_mask(fields[23])?;
    let can_flee: bool = fields[25].parse()?;
    let can_call: bool = fields[24].parse()?;
    let friendly_prob: u32 = fields[26].parse()?;
    let count_in_group_expr = fields[27].to_owned();
    let follower = parse_follower(fields[29], fields[28])?;
    let xp_expr = fields[6].to_owned();
    let is_invincible: bool = fields[39].parse()?;
    let attack_twice: bool = fields[40].parse()?;
    let description = fields[45].to_owned();
    let hide_in_catalog: bool = fields[48].parse()?;

    Ok(Monster {
        id,
        name_ident,
        name_unident,
        name_plural_ident,
        name_plural_unident,
        kind,
        xl_expr,
        hp_expr,
        mp_expr,
        ac_expr,
        stats,
        damage_expr,
        attack_count_expr,
        attack_debuff_mask,
        poison_damage,
        drain_xl,
        spell_levels,
        healing,
        resist_mask,
        spell_cancel,
        vuln_mask,
        can_flee,
        can_call,
        friendly_prob,
        count_in_group_expr,
        follower,
        xp_expr,
        is_invincible,
        attack_twice,
        description,
        hide_in_catalog,
    })
}

fn parse_attack_debuff_mask(s: &str) -> anyhow::Result<DebuffMask> {
    let mut bits = 0;

    for c in s.chars() {
        let effect = c
            .to_digit(10)
            .with_context(|| format!("invalid attack effect char: {}", c))?;

        bits |= 1 << effect;
    }

    let mask = DebuffMask::from_bits(bits)
        .with_context(|| format!("unknown debuff mask bit: {:#b}", bits))?;

    Ok(mask)
}

/// util::parse_resist_mask() とは異なる。
/// モンスターの抵抗/弱点マスクは bit 配列が異なるため、変換が必要。
fn parse_resist_mask(s: &str) -> anyhow::Result<ResistMask> {
    // (bit位置, 属性)
    const TRANSLATION: &[(u8, ResistMask)] = &[
        (0, ResistMask::SLEEP),
        (1, ResistMask::KNOCKOUT),
        (2, ResistMask::CRITICAL),
        (3, ResistMask::DEATH),
        (4, ResistMask::FIRE),
        (5, ResistMask::COLD),
        (6, ResistMask::ELECTRIC),
        (7, ResistMask::HOLY),
        (8, ResistMask::GENERIC),
        (9, ResistMask::SILENCE),
        (10, ResistMask::POISON),
        (11, ResistMask::PARALYSIS),
        (12, ResistMask::PETRIFICATION),
    ];

    let mut bits = 0;

    for c in s.chars() {
        let element = c
            .to_digit(16)
            .with_context(|| format!("invalid element char: {}", c))?;

        bits |= 1 << element;
    }

    let mut mask = ResistMask::empty();

    for &(i, mask_elem) in TRANSLATION {
        if (bits & (1 << i)) != 0 {
            mask |= mask_elem;
        }
    }

    Ok(mask)
}

fn parse_follower(s_id: &str, s_prob: &str) -> anyhow::Result<Option<MonsterFollower>> {
    if s_id.is_empty() {
        return Ok(None);
    }

    let id_expr = s_id.to_owned();

    let prob: u32 = if s_prob.is_empty() {
        50
    } else {
        s_prob.parse()?
    };

    Ok(Some(MonsterFollower { id_expr, prob }))
}
