use std::ops::RangeInclusive;
use bevy::{prelude::{Vec3, Res, Query, Component}, time::{Time, Timer}};
use itertools::iproduct;
use ourcraft::{Blocs, Pos, BlocPos};
const SPEED: f32 = 20.;
const ACC: f32 = 10.;

#[derive(Component)]
pub struct Jumping {
    pub force: f32,
    pub cd: Timer,
    pub intent: bool,
}

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
    (v.floor() as i32)..=((size+v).floor() as i32)
}

fn blocs_perp_y(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.x, aabb.0.x) , extent(pos.z, aabb.0.z)).map(move |(x, z)| Pos {
        x, y: pos.y.floor() as i32, z, realm: pos.realm
    })
}

fn blocs_perp_z(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.x, aabb.0.x) , extent(pos.y, aabb.0.y)).map(move |(x, y)| Pos {
        x, y, z: pos.z.floor() as i32, realm: pos.realm
    })
}

fn blocs_perp_x(pos: Pos<f32>, aabb: &AABB) -> impl Iterator<Item = BlocPos> {
    iproduct!(extent(pos.y, aabb.0.y) , extent(pos.z, aabb.0.z)).map(move |(y, z)| Pos {
        x: pos.x.floor() as i32, y, z, realm: pos.realm
    })
}

pub fn process_jumps(blocs: Res<Blocs>, time: Res<Time>, mut query: Query<(&Pos, &AABB, &mut Jumping, &mut Velocity)>) {
    for (pos, aabb, mut jumping, mut velocity) in query.iter_mut() {
        jumping.cd.tick(time.delta());
        if jumping.intent && jumping.cd.finished() {
            let below = *pos + Vec3::new(0., -1., 0.);
            if blocs_perp_y(below, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                velocity.0.y += jumping.force;
                jumping.cd.reset();
            }
        }
    }
}

pub fn apply_acc(
    blocs: Res<Blocs>,
    time: Res<Time>,
    mut query: Query<(&Heading, &mut Velocity, &Pos, &AABB)>
) {
    for (heading, mut velocity, pos, aabb) in query.iter_mut() {
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
        // make velocity inch towards heading
        let speed = heading.length();
        let diff = heading-velocity.0;
        let mut acc: Vec3 = (ACC*time.delta_seconds()).min(1.)*diff/(diff.length()*speed*friction).max(1.);
        if acc.y.is_nan() {
            acc.y = 0.;
        }
        velocity.0 += acc;
    }
}

pub fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.0 += Vec3::new(0., -gravity.0*time.delta_seconds(), 0.);
    }
}

pub fn apply_speed(blocs: Res<Blocs>, time: Res<Time>, mut query: Query<(&mut Velocity, &mut Pos, &AABB)>) {
    for (mut velocity, mut pos, aabb) in query.iter_mut() {
        let applied_velocity = velocity.0*time.delta_seconds()*SPEED;
        let clamped_velocity = applied_velocity.clamp(Vec3::new(-1., -1., -1.), Vec3::new(1., 1., 1.));
        // split the motion on all 3 axis, check for collisions, adjust the final speed vector if there's any
        // x
        if applied_velocity.x != 0. {
            let sx = clamped_velocity.x;
            let dx = if sx > 0. { aabb.0.x + sx } else { sx };
            let pos_x = *pos + Vec3::new(dx, 0., 0.);
            if blocs_perp_x(pos_x, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if sx > 0. { 
                    pos.x = pos.x.ceil() - aabb.0.x - f32::EPSILON;
                } else { 
                    pos.x = pos.x.floor() + f32::EPSILON;
                }
                velocity.0.x = 0.;
            } else {
                pos.x += applied_velocity.x;
            }
        }
        // y
        if applied_velocity.y != 0. {
            let sy = clamped_velocity.y;
            let dy = if sy > 0. { aabb.0.y + sy } else { sy };
            let pos_y = *pos + Vec3::new(0., dy, 0.);
            if blocs_perp_y(pos_y, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if sy > 0. { 
                    pos.y = pos.y.ceil() - aabb.0.y - f32::EPSILON;
                } else { 
                    pos.y = pos.y.floor() + f32::EPSILON;
                }
                velocity.0.y = 0.;
            } else {
                pos.y += applied_velocity.y;
            }
        }
        // z
        if applied_velocity.z != 0. {
            let sz = clamped_velocity.z;
            let dz: f32 = if sz > 0. { aabb.0.z + sz } else { sz };
            let pos_z = *pos + Vec3::new(0., 0., dz);
            if blocs_perp_z(pos_z, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if sz > 0. { 
                    pos.z = pos.z.ceil() - aabb.0.z - f32::EPSILON;
                } else { 
                    pos.z = pos.z.floor() + f32::EPSILON;
                }
                velocity.0.z = 0.;
            } else {
                pos.z += applied_velocity.z;
            }
        }
    }
}