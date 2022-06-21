pub struct CacheUpdates<T> {
    invalidated: Vec<T>,
    new_entries: Vec<T>,
}

impl<T> CacheUpdates<T> {
    pub fn new() -> Self{
        CacheUpdates {
            invalidated: Vec::new(),
            new_entries: Vec::new(),
        }
    }

    /*pub fn new(invalidated: Vec<T>, new_entries: Vec<T>) -> Self {
        Self { invalidated, new_entries }
    }*/

    pub fn add_invalidated(&mut self, item: T) {
        self.invalidated.push(item);
    }

    pub fn add_new(&mut self, item: T) {
        self.new_entries.push(item);
    }

    pub fn invalidated(&self) -> &Vec<T> {
        &self.invalidated
    }

    pub fn new_entries(&self) -> &Vec<T> {
        &self.new_entries
    }
}