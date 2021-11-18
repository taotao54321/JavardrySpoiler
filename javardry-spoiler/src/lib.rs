pub mod cipher;
mod class;
mod item;
mod kvs;
mod monster;
mod race;
mod scenario;
mod spell;
mod stat;
mod util;

pub use crate::class::*;
pub use crate::item::*;
pub use crate::monster::*;
pub use crate::race::*;
pub use crate::scenario::*;
pub use crate::spell::*;
pub use crate::stat::*;

use bitflags::bitflags;

bitflags! {
    pub struct ResistMask: u32 {
        const SILENCE = 1 << 0;
        const SLEEP = 1 << 1;
        const POISON = 1 << 2;
        const PARALYSIS = 1 << 3;
        const PETRIFICATION = 1 << 4;
        const DRAIN = 1 << 5;
        const KNOCKOUT = 1 << 6;
        const CRITICAL = 1 << 7;
        const DEATH = 1 << 8;
        // XXX: bit9 は未使用?
        const FIRE = 1 << 10;
        const COLD = 1 << 11;
        const ELECTRIC = 1 << 12;
        const HOLY = 1 << 13;
        const GENERIC = 1 << 14; // 無属性
    }
}

bitflags! {
    pub struct DebuffMask: u32 {
        const SLEEP = 1 << 0;
        const PARALYSIS = 1 << 1;
        const PETRIFICATION = 1 << 2;
        const KNOCKOUT = 1 << 3;
        const CRITICAL = 1 << 4;
    }
}
