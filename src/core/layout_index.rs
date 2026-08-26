use generational_arena::Index;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutIndex {
    Existing(Index),
    Empty(usize),
}