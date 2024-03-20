pub trait GetSet<V>: Sized {
    fn get(&self, i: usize) -> V;

    fn set(&mut self, i: usize, value: V);
}

impl<V: Copy> GetSet<V> for Vec<V> {
    fn get(&self, i: usize) -> V {
        self[i]
    }

    fn set(&mut self, i: usize, value: V) {
        self[i] = value;
    }
}
