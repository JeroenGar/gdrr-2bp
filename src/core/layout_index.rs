use slotmap::new_key_type;

new_key_type! {
    pub struct LayoutKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutIndex {
    Existing(LayoutKey),
    Empty(u32),
}

impl LayoutIndex {
    pub fn empty(index: usize) -> Self {
        Self::Empty(u32::try_from(index).expect("problem exceeds u32 empty-layout indices"))
    }
}
