
pub trait GameEntityKey:
    Send + Sync + 'static + PartialEq + Clone + Copy + PartialOrd + std::fmt::Debug + Eq + Ord + std::hash::Hash
{
}

//todo blanket impl

impl GameEntityKey for () {}
impl GameEntityKey for &'static str {}
impl GameEntityKey for u8 {}
impl GameEntityKey for u16 {}
impl GameEntityKey for u32 {}
impl GameEntityKey for u64 {}
