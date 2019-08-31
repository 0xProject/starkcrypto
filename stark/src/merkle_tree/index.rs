/// Index into a balances binary tree
///
/// 
/// Nodes are indexed [0...n-1], where n = 2^k-1 is the total number of leafs
/// and nodes in the tree. Nodes are indexed in breadth-first order, starting
/// with the root at 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(usize);

// TODO: Shift the internal representation by one.
impl Index {
    pub const fn root() -> Self {
        Self(0)
    }
    pub fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub fn from_depth_offset(depth: usize, offset: usize) -> Self {
        // At level `depth` there are 2^depth nodes at offsets [0..2^depth-1]
        assert!(offset < 1usize << depth);
        Self((1usize << depth) - 1 + offset)
    }

    pub fn index(&self) -> usize {
        self.0
    }

    pub fn depth(&self) -> usize {
        ((self.0 + 2).next_power_of_two().trailing_zeros() as usize) - 1
    }

    pub fn offset(&self) -> usize {
        (self.0 + 1) - (self.0 + 1).next_power_of_two()
    }

    pub fn is_root(&self) -> bool {
        self.0 == 0
    }

    pub fn is_left(&self) -> bool {
        self.0 % 2 == 1
    }

    pub fn is_right(&self) -> bool {
        self.0 != 0 && self.0 % 2 == 0
    }

    pub fn is_left_most(&self) -> bool {
        (self.0 + 1).is_power_of_two()
    }

    pub fn is_right_most(&self) -> bool {
        (self.0 + 2).is_power_of_two()
    }

    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            None
        } else {
            Some(Self((self.0 - 1) >> 1))
        }
    }

    pub fn sibling(&self) -> Option<Self> {
        if self.is_root() {
            None
        } else if self.is_left() {
            Some(Self(self.0 + 1))
        } else {
            Some(Self(self.0 - 1))
        }
    }

    pub fn left_neighbor(&self) -> Option<Self> {
        if self.is_left_most() {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }

    pub fn right_neighbor(&self) -> Option<Self> {
        if self.is_right_most() {
            None
        } else {
            Some(Self(self.0 + 1))
        }
    }

    pub fn left_child(&self) -> Self {
        Self(2 * self.0 + 1)
    }

    pub fn right_child(&self) -> Self {
        Self(2 * self.0 + 2)
    }

    pub fn ancestor_of(&self, other: Index) -> bool {

    }

    pub fn descents_from(&self, other: ) -> bool {

    }

    pub fn last_common_ancestor(&self, other: Self) -> Self {
        // TODO
    }
}