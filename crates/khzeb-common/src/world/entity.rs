use core::fmt;

pub type EntityGen = u32;
pub type EntityIdx = u32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(u64);

impl Entity {
    pub fn new(generation: EntityGen, index: EntityIdx) -> Self {
        Self(((generation as u64) << 32) | index as u64)
    }

    pub fn generation(&self) -> EntityGen {
        ((self.0 >> 32) & 0xFFFFFFFF) as EntityGen
    }

    pub fn index(&self) -> EntityIdx {
        (self.0 & 0xFFFFFFFF) as EntityIdx
    }

    pub fn decouple(&self) -> (EntityGen, EntityIdx) {
        (self.generation(), self.index())
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<Entity gen={} idx={}>",
            self.generation(),
            self.index()
        ))
    }
}
