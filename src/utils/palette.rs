pub trait Palette<E> {
    fn index(&mut self, elem: E) -> usize;
}

impl<E: Eq> Palette<E> for Vec<E> {
    fn index(&mut self, elem: E) -> usize {
        self.iter().position(|other| *other == elem).unwrap_or({
            // Element is not present in the palette
            self.push(elem);
            self.len() - 1
        })
    }
}