use itertools::Itertools as _;

use javardry_spoiler::{
    Class, DebuffMask, ItemKind, MonsterKind, MonsterKindMask, Race, ResistMask, Scenario,
};

pub(crate) fn strip_text_tags(s: impl AsRef<str>) -> String {
    let s = s.as_ref();

    s.replace("<br>", "")
}

pub(crate) fn bool_str(b: bool) -> String {
    if b { "o" } else { "" }.to_owned()
}

pub(crate) fn resist_mask_str(mask: ResistMask) -> String {
    const TABLE: &[(ResistMask, char)] = &[
        (ResistMask::SILENCE, '黙'),
        (ResistMask::SLEEP, '眠'),
        (ResistMask::POISON, '毒'),
        (ResistMask::PARALYSIS, '麻'),
        (ResistMask::PETRIFICATION, '石'),
        (ResistMask::DRAIN, '吸'),
        (ResistMask::KNOCKOUT, '気'),
        (ResistMask::CRITICAL, '首'),
        (ResistMask::DEATH, '死'),
        (ResistMask::FIRE, '火'),
        (ResistMask::COLD, '冷'),
        (ResistMask::ELECTRIC, '電'),
        (ResistMask::HOLY, '聖'),
        (ResistMask::GENERIC, '無'),
    ];

    let mut res = "".to_owned();

    for &(mask_elem, c) in TABLE {
        if mask.contains(mask_elem) {
            res.push(c);
        }
    }

    res
}

pub(crate) fn debuff_mask_str(mask: DebuffMask) -> String {
    const TABLE: &[(DebuffMask, char)] = &[
        (DebuffMask::SLEEP, '眠'),
        (DebuffMask::PARALYSIS, '麻'),
        (DebuffMask::PETRIFICATION, '石'),
        (DebuffMask::KNOCKOUT, '気'),
        (DebuffMask::CRITICAL, '首'),
    ];

    let mut res = "".to_owned();

    for &(mask_elem, c) in TABLE {
        if mask.contains(mask_elem) {
            res.push(c);
        }
    }

    res
}

pub(crate) fn sex_mask_str(mask: u8) -> String {
    const CHARS: &[char] = &['男', '女'];

    let mut res = "".to_owned();

    for (i, &c) in CHARS.iter().enumerate() {
        if (mask & (1 << i)) != 0 {
            res.push(c);
        }
    }

    res
}

pub(crate) fn alignment_mask_str(mask: u8) -> String {
    const CHARS: &[char] = &['G', 'N', 'E'];

    let mut res = "".to_owned();

    for (i, &c) in CHARS.iter().enumerate() {
        if (mask & (1 << i)) != 0 {
            res.push(c);
        }
    }

    res
}

pub(crate) fn item_kind_str(kind: ItemKind) -> String {
    match kind {
        ItemKind::Weapon => "武器",
        ItemKind::Armor => "鎧",
        ItemKind::Shield => "盾",
        ItemKind::Helmet => "兜",
        ItemKind::Gloves => "小手",
        ItemKind::Boots => "靴",
        ItemKind::Tool => "道具",
    }
    .to_owned()
}

pub(crate) fn race_mask_str(scenario: &Scenario, mask: u64) -> String {
    fn race_char(race: &Race) -> char {
        race.name_abbr.chars().next().unwrap_or('?')
    }

    scenario
        .races
        .iter()
        .enumerate()
        .map(|(i, race)| {
            if (mask & (1 << i)) != 0 {
                race_char(race)
            } else {
                '-'
            }
        })
        .collect()
}

pub(crate) fn class_mask_str(scenario: &Scenario, mask: u64) -> String {
    fn class_char(class: &Class) -> char {
        class.name_abbr.chars().next().unwrap_or('?')
    }

    scenario
        .classes
        .iter()
        .enumerate()
        .map(|(i, class)| {
            if (mask & (1 << i)) != 0 {
                class_char(class)
            } else {
                '-'
            }
        })
        .collect()
}

pub(crate) fn monster_kind_str(kind: MonsterKind) -> String {
    match kind {
        MonsterKind::Fighter => "戦士",
        MonsterKind::Mage => "魔法使い",
        MonsterKind::Priest => "僧侶",
        MonsterKind::Thief => "盗賊",
        MonsterKind::Midget => "小人",
        MonsterKind::Giant => "巨人",
        MonsterKind::Myth => "神話",
        MonsterKind::Dragon => "竜",
        MonsterKind::Animal => "動物",
        MonsterKind::Werecreature => "獣人",
        MonsterKind::Undead => "不死",
        MonsterKind::Demon => "悪魔",
        MonsterKind::Insect => "昆虫",
        MonsterKind::Enchanted => "魔法生物",
        MonsterKind::Mystery => "謎の生物",
    }
    .to_owned()
}

pub(crate) fn monster_kind_mask_str(mask: MonsterKindMask) -> String {
    let bits = mask.bits();

    (0..u8::try_from(u32::BITS).unwrap())
        .filter_map(|i| {
            ((bits & (1 << i)) != 0).then(|| {
                monster_kind_str(
                    MonsterKind::try_from(i).expect("monster kind value should be valid"),
                )
            })
        })
        .join(" ")
}
