use itertools::Itertools;

pub(crate) trait Counter<E> {
    fn add(&mut self, elem: E)
    where E: PartialEq<E>;

    fn divide(&mut self, value: f32);

    fn ordered(&mut self);
}

impl<E> Counter<E> for Vec<(E, f32)> {
    fn add(&mut self, elem: E)
        where E: PartialEq<E> 
    {
        if let Some((i, (_, count))) = self.iter().find_position(|(e, _)| *e == elem) {
            self[i] = (elem, *count+1.);
        } else {
            self.push((elem, 1.));
        }
    }

    fn divide(&mut self, value: f32) {
        for i in 0..self.len() {
            self[i].1 /= value;
        }
    }

    fn ordered(&mut self) {
        self.sort_by(|(_, c1), (_, c2)| c2.partial_cmp(c1).unwrap());
    }
}