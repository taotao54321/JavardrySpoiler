use anyhow::{anyhow, ensure};

use crate::kvs::{Kvs, KvsExt};

/// 特性値。
#[derive(Debug)]
pub struct Stat {
    pub id: u32,
    pub name: String,
    pub name_abbr: String,
    pub sex_bonus: [i32; 2],
    pub fixed_on_create: bool, // キャラ作成時にボーナスポイントを振れない
    pub hide: bool,
    // TODO: 最大値(色々面倒なので保留)
}

pub(crate) fn stats_from_kvs(kvs: &Kvs) -> anyhow::Result<Vec<Stat>> {
    let mut stats = Vec::<Stat>::new();

    for (i, text) in kvs.iter_seq("Abi").enumerate() {
        let id = u32::try_from(i).expect("stat id should be u32");
        let stat = parse(id, text).map_err(|e| anyhow!("stat {}: {}", id, e))?;
        stats.push(stat);
    }

    Ok(stats)
}

fn parse(id: u32, text: impl AsRef<str>) -> anyhow::Result<Stat> {
    let text = text.as_ref();

    let fields: Vec<_> = text.split("<>").collect();
    ensure!(fields.len() == 8, "stat text must have 8 fields");

    let name = fields[0].to_owned();
    let name_abbr = fields[1].to_owned();
    let sex_bonus: [i32; 2] = [fields[2].parse()?, fields[3].parse()?];
    let fixed_on_create: bool = fields[4].parse()?;
    let hide: bool = fields[7].parse()?;

    Ok(Stat {
        id,
        name,
        name_abbr,
        sex_bonus,
        fixed_on_create,
        hide,
    })
}
