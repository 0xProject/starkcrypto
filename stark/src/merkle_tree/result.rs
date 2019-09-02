pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    NumLeavesNotPowerOfTwo,
    DepthOutOfRange,
    IndexOutOfRange,
    IndicesUnsortedOrDuplicate,
    DuplicateLeafMismatch,
    NotEnoughHashes,
    RootHashMismatch,
}