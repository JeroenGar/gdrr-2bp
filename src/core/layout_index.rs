use generational_arena::Index;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LayoutIndex {
    Existing(Index),
    Empty(usize),
}