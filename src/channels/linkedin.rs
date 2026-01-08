pub struct PlaceholderProvider;

impl PlaceholderProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlaceholderProvider {
    fn default() -> Self {
        Self::new()
    }
}
