use slotmap::new_key_type;

new_key_type! {
    pub struct LayoutKey;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutIndex {
    Existing(LayoutKey),
    Empty(usize),
}
