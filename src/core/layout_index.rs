use generational_arena::Index;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LayoutIndex {
    Existing(Index),
    Empty(usize),
}

impl LayoutIndex{
    fn to_index(&self) -> Index {
        match self {
            LayoutIndex::Existing(index) => *index,
            LayoutIndex::Empty(index) => panic!("Cannot convert empty layout index to index"),
        }
    }
}