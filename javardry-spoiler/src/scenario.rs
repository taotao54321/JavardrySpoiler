use crate::class::{classes_from_kvs, Class};
use crate::item::{items_from_kvs, Item};
use crate::kvs::KvsExt;
use crate::monster::{monsters_from_kvs, Monster};
use crate::race::{races_from_kvs, Race};
use crate::spell::{spell_realms_from_kvs, SpellRealm};
use crate::stat::{stats_from_kvs, Stat};

#[derive(Debug)]
pub struct Scenario {
    pub editor_version: String,
    pub id: String,
    pub title: String,
    pub stats: Vec<Stat>,
    pub races: Vec<Race>,
    pub classes: Vec<Class>,
    pub spell_realms: Vec<SpellRealm>,
    pub items: Vec<Item>,
    pub monsters: Vec<Monster>,
}

impl Scenario {
    pub fn load_from_ciphertext(ciphertext: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let plaintext = crate::cipher::decrypt(ciphertext)?;

        Self::load_from_plaintext(plaintext)
    }

    pub fn load_from_plaintext(plaintext: impl AsRef<str>) -> anyhow::Result<Self> {
        let kvs = crate::kvs::parse(plaintext)?;

        let editor_version = kvs.get_expect("Version")?.to_owned();
        let id = kvs.get_expect("ReadKeyword")?.to_owned();
        let title = kvs.get_expect("GameTitle")?.to_owned();
        let stats = stats_from_kvs(&kvs)?;
        let races = races_from_kvs(&kvs)?;
        let classes = classes_from_kvs(&kvs)?;
        let spell_realms = spell_realms_from_kvs(&kvs)?;
        let items = items_from_kvs(&kvs)?;
        let monsters = monsters_from_kvs(&kvs)?;

        Ok(Self {
            editor_version,
            id,
            title,
            stats,
            races,
            classes,
            spell_realms,
            items,
            monsters,
        })
    }
}
