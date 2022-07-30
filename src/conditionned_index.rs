use itertools::zip;
use vpsearch;

#[derive(Clone, Copy)]
struct Condition<const D: usize> {
    values: [f32; D],
    pickiness: f32,
}

impl<const D: usize> vpsearch::MetricSpace for Condition<D> {
    type UserData = ();
    type Distance = f32;

    fn distance(&self, other: &Self, _: &Self::UserData) -> Self::Distance {
        let mut res = 0.;
        for (a, b) in zip(&self.values, &other.values) {
            res += (a - b).powi(2);
        }
        res.sqrt() * self.pickiness * other.pickiness
    }
}

impl<E, const D: usize> From<(E, [f32; D], f32)> for Condition<D> {
    fn from(conditionned: (E, [f32; D], f32)) -> Self {
        Condition {
            values: conditionned.1,
            pickiness: conditionned.2,
        }
    }
}

impl<E, const D: usize> From<(E, [f32; D])> for Condition<D> {
    fn from(conditionned: (E, [f32; D])) -> Self {
        Condition {
            values: conditionned.1,
            ..Default::default()
        }
    }
}

impl<const D: usize> Default for Condition<D> {
    fn default() -> Self {
        Self {
            values: [0.; D],
            pickiness: 1.,
        }
    }
}

pub struct ConditionnedIndex<E, const D: usize> {
    elems: Vec<E>,
    search: vpsearch::Tree<Condition<D>>,
}

impl<E: Clone, const D: usize> ConditionnedIndex<E, D> {
    pub fn new(data: Vec<(E, [f32; D], f32)>) -> Self {
        ConditionnedIndex {
            search: vpsearch::Tree::new(
                &(data.clone())
                    .into_iter()
                    .map(Condition::from)
                    .collect::<Vec<Condition<D>>>(),
            ),
            elems: data.into_iter().map(|(e, _, _)| e).collect(),
        }
    }

    pub fn with_default_pickiness(data: Vec<(E, [f32; D])>) -> Self {
        ConditionnedIndex {
            search: vpsearch::Tree::new(
                &(data.clone())
                    .into_iter()
                    .map(Condition::from)
                    .collect::<Vec<Condition<D>>>(),
            ),
            elems: data.into_iter().map(|(e, _)| e).collect(),
        }
    }

    pub fn closest(&self, point: &[f32; D]) -> (E, f32) {
        let (i, dist) = self.search.find_nearest(&Condition {
            values: *point,
            ..Default::default()
        });
        (self.elems[i].clone(), dist)
    }
}
