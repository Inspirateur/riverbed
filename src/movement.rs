use std::ops::RangeInclusive;
use bevy::{prelude::{Vec3, Res, Query, Component}, time::Time};
use itertools::iproduct;
use ourcraft::{Blocs, Pos, BlocPos};
const SPEED: f32 = 20.;

#[derive(Component)]
pub struct AABB(pub Vec3);

// both describe the speed and the direction the entity wants to go towards
#[derive(Component)]
pub struct Heading(pub Vec3);

// the actual speed vector of the entity, most of the time it is similar to Heading but not always!
#[derive(Component)]
pub struct Velocity(pub Vec3);

// the gravity that the entity will be subject to
#[derive(Component)]
pub struct Gravity(pub f32);

fn extent(v: f32, size: f32) -> RangeInclusive<i32> {
    (v as i32)..=((size+v) as i32)
}

fn blocs_perp_y(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.x, aabb.0.x) , extent(pos.z, aabb.0.z)).map(move |(x, z)| Pos {
        x, y: pos.y as i32, z, realm: pos.realm
    })
}

fn blocs_perp_z(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.x, aabb.0.x) , extent(pos.y, aabb.0.y)).map(move |(x, y)| Pos {
        x, y, z: pos.z as i32, realm: pos.realm
    })
}

fn blocs_perp_x(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.y, aabb.0.y) , extent(pos.z, aabb.0.z)).map(move |(y, z)| Pos {
        x: pos.x as i32, y, z, realm: pos.realm
    })
}

pub fn apply_acc(
    blocs: Res<Blocs>,
    time: Res<Time>,
    mut query: Query<(&Heading, &mut Velocity, &Pos, &Gravity, &AABB)>
) {
    for (heading, mut velocity, pos, gravity, aabb) in query.iter_mut() {
        // get the bloc the entity is standing on if the entity has an AABB
        let mut friction: f32 = 0.;
        let mut slowing: f32 = 0.;
        let below = *pos + Vec3::new(0., -1., 0.);
        for bloc in blocs_perp_y(below, aabb).map(|blocpos| blocs.get_block(blocpos)) {
            friction = friction.max(bloc.slowing());
            slowing = slowing.max(bloc.slowing())
        }
        // applying slowing
        let heading = heading.0*slowing;
        println!("heading: {}", heading);
        // make velocity inch towards heading
        let speed = heading.length();
        let diff = heading-velocity.0;
        let acc = time.delta_seconds()*diff/(diff.length()*speed*friction).max(1.);
        velocity.0 += acc + Vec3::new(0., -1., 0.)*gravity.0*time.delta_seconds();
        println!("velocity: {}", velocity.0);
    }
}

pub fn apply_speed(blocs: Res<Blocs>, time: Res<Time>, mut query: Query<(&Velocity, &mut Pos, &AABB)>) {
    for (velocity, mut pos, aabb) in query.iter_mut() {
        // split the motion on all 3 axis, check for collisions, adjust the final speed vector if there's any
        // x
        let sx = velocity.0.x.signum();
        let dx = if sx > 0. { aabb.0.x + sx } else { sx };
        let pos_x = *pos + Vec3::new(dx, 0., 0.);
        pos.x += if blocs_perp_x(pos_x, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
            // there's a collision in this direction, stop at the block limit
            if sx > 0. { 1. - pos_x.x.fract() - f32::EPSILON } else { - pos_x.x.fract() + f32::EPSILON }
        } else {
            velocity.0.x
        }*time.delta_seconds()*SPEED;
        // y
        let sy = velocity.0.y.signum();
        let dy = if sy > 0. { aabb.0.y + sy } else { sy };
        let pos_y = *pos + Vec3::new(0., dy, 0.);
        pos.y += if blocs_perp_y(pos_y, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
            // there's a collision in this direction, stop at the block limit
            if sy > 0. { 1. - pos_y.y.fract() - f32::EPSILON } else { - pos_y.y.fract() + f32::EPSILON }
        } else {
            velocity.0.y
        }*time.delta_seconds()*SPEED;
        // z
        let sz = velocity.0.z.signum();
        let dz = if sz > 0. { aabb.0.z + sz } else { sz };
        let pos_z = *pos + Vec3::new(0., 0., dz);
        pos.z += if blocs_perp_z(pos_z, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
            // there's a collision in this direction, stop at the block limit
            if sz > 0. { 1. - pos_z.z.fract() - f32::EPSILON } else { - pos_z.z.fract() + f32::EPSILON }
        } else {
            velocity.0.z
        }*time.delta_seconds()*SPEED;
    }
}