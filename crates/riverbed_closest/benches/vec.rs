use std::ops::Range;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use riverbed_closest::{points, ranges, ClosestTrait};

fn range_closest(c: &mut Criterion) {
    let imap: Vec<([Range<f32>; 4], String)> = ranges::from_csv("benches/plants_ranges.csv").unwrap();
    let point = core::array::from_fn(|_| 0.5);
    c.bench_function(&format!("range-closest ({})", imap.len()), |b| b.iter(|| 
        black_box(imap.closest(point))
    ));
}

fn point_closest(c: &mut Criterion) {
    let imap: Vec<([f32; 4], String)> = points::from_csv("benches/plants_points.csv").unwrap();
    let point = core::array::from_fn(|_| 0.5);
    c.bench_function(&format!("point-closest ({})", imap.len()), |b| b.iter(|| 
        black_box(imap.closest(point))
    ));
}

criterion_group!(vec, range_closest, point_closest);
criterion_main!(vec);