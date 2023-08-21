use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::iproduct;
use nd_interval::NdInterval;
use noise_algebra::NoiseSource;
use ourcraft::{MAX_HEIGHT, CHUNK_S1, Bloc, Soils, Col};

fn col_gen(n: &mut NoiseSource<2>, soils: &Soils) {
    let mut col = Col::new();
    let cont = (n.simplex(0.7) + n.simplex(3.) * 0.3).normalize();
    let land = cont.clone() + n.simplex(9.) * 0.1;
    let ocean = !(cont*0.5 + 0.5);
    let land = land.normalize().mask(0.35);
    let mount_mask = (n.simplex(1.) + n.simplex(2.)*0.3).normalize().mask(0.2)*land.clone();
    let mount = (!n.simplex(0.8).powi(2) + n.simplex(1.5).powi(2)*0.4).normalize() * mount_mask;
    // WATER_R is used to ensure land remains above water even if water level is raised
    let ys = 0.009 + land*0.3 + mount*(1.-0.3);
    // more attitude => less temperature
    let ts = !ys.clone().powi(3) * (n.simplex(0.2)*0.5 + 0.5 + n.simplex(0.6)*0.3).normalize();
    // closer to the ocean => more humidity
    // higher temp => more humidity
    let hs = (ocean + ts.clone().powf(0.5) * (n.simplex(0.5)*0.5 + 0.5)).normalize();
    for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
        let (y, t, h) = (ys[i], ts[i], hs[i]);
        let y = (y * MAX_HEIGHT as f64) as i32;
        assert!(y >= 0);
        let bloc = match soils.closest([t as f32, h as f32]) {
            Some((bloc, _)) => *bloc,
            None => Bloc::Dirt,
        };
        col.set((dx, y, dz), bloc);
        for y_ in (y-3)..y {
            if y_ < 0 {
                break;
            }
            col.set((dx, y_, dz), Bloc::Dirt);
        }
    }
}

fn col_gen_bench(c: &mut Criterion) {
    let mut n = NoiseSource::new([0..=(CHUNK_S1 as i32-1), 0..=(CHUNK_S1 as i32-1)], 42, 1);
    let soils = Soils::new();
    c.bench_function(&format!("col_gen-{}^2", CHUNK_S1), |b| b.iter(|| 
        col_gen(black_box(&mut n), black_box(&soils))
    ));
}

criterion_group!(generation, col_gen_bench);
criterion_main!(generation);