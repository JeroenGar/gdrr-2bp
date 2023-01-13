use generational_arena::Index;

#[derive(Debug, Clone, Copy)]
pub enum LayoutIndex {
    Existing(Index),
    Empty(usize),
}