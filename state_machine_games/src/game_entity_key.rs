pub trait GameEntityKey:
    Send
    + Sync
    + 'static
    + PartialEq
    + Clone
    + Copy
    + PartialOrd
    + std::fmt::Debug
    + Eq
    + Ord
    + std::hash::Hash
{
}

impl<
    T: Send
        + Sync
        + 'static
        + PartialEq
        + Clone
        + Copy
        + PartialOrd
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash,
> GameEntityKey for T
{
}
