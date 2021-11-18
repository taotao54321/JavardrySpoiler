use anyhow::{anyhow, ensure};

use crate::kvs::{Kvs, KvsExt};
use crate::util;
use crate::ResistMask;

#[derive(Debug)]
pub struct Race {
    pub id: u32,
    pub name: String,
    pub name_abbr: String,
    pub stats: Vec<u32>,
    pub lifetime: u32,
    pub ac: i32,
    pub healing: i32,
    pub spell_cancel: i32,
    pub resist_mask: ResistMask,
    pub cond_to_appear: String,
    pub description: String,
    pub inven_bonus: i32,
    // TODO: ブレス関連
}

pub(crate) fn races_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<Race>> {
    let mut races = Vec::<Race>::new();

    for (i, text) in kvs.iter_seq("Race").enumerate() {
        let id = u32::try_from(i).expect("race id should be u32");
        let race = parse(id, text).map_err(|e| anyhow!("race {}: {}", id, e))?;
        races.push(race);
    }

    Ok(races)
}

fn parse(id: u32, text: impl AsRef<str>) -> anyhow::Result<Race> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<>").collect();
    ensure!(fields.len() == 14, "race text must have 14 fields");

    let name = fields[0].to_owned();
    let name_abbr = fields[1].to_owned();
    let stats: Vec<u32> = fields[2]
        .split(',')
        .map(str::parse::<u32>)
        .collect::<Result<_, _>>()?;
    let lifetime: u32 = fields[3].parse()?;
    let ac: i32 = fields[4].parse()?;
    let healing: i32 = fields[5].parse()?;
    let spell_cancel: i32 = fields[6].parse()?;
    let resist_mask = util::parse_resist_mask(fields[9])?;
    let cond_to_appear = fields[10].to_owned();
    let description = fields[11].to_owned();
    let inven_bonus: i32 = fields[13].parse()?;

    Ok(Race {
        id,
        name,
        name_abbr,
        stats,
        lifetime,
        ac,
        healing,
        spell_cancel,
        resist_mask,
        cond_to_appear,
        description,
        inven_bonus,
    })
}
