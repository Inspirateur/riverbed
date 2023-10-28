use bevy::{prelude::*, time::{Time, Timer}};
use itertools::{iproduct, Itertools};
use crate::blocs::{Blocs, Pos, BlocPos};
const SPEED: f32 = 30.;
const ACC: f32 = 15.;

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

fn extent(v: f32, size: f32) -> Vec<i32> {
    if size > 0. {
        ((v.floor() as i32)..=((size+v).floor() as i32)).collect_vec()
    } else {
        (((size+v).floor() as i32)..=(v.floor() as i32)).rev().collect_vec()
    }
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
            let below = *pos + Vec3::new(0., -0.01, 0.);
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
        if !blocs.is_col_loaded(*pos) {
            continue;
        }
        // get the bloc the entity is standing on if the entity has an AABB
        let mut friction: f32 = 0.;
        let mut slowing: f32 = 0.;
        let below = *pos + Vec3::new(0., -0.001, 0.);
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

pub fn apply_gravity(blocs: Res<Blocs>, time: Res<Time>, mut query: Query<(&Pos, &mut Velocity, &Gravity)>) {
    for (pos, mut velocity, gravity) in query.iter_mut() {
        if !blocs.is_col_loaded(*pos) {
            continue;
        }
        velocity.0 += Vec3::new(0., -gravity.0*time.delta_seconds(), 0.);
    }
}

pub fn apply_speed(blocs: Res<Blocs>, time: Res<Time>, mut query: Query<(&mut Velocity, &mut Pos, &AABB)>) {
    for (mut velocity, mut pos, aabb) in query.iter_mut() {
        if !blocs.is_col_loaded(*pos) {
            continue;
        }
        let applied_velocity = velocity.0*time.delta_seconds()*SPEED;
        // split the motion on all 3 axis, check for collisions, adjust the final speed vector if there's any
        // x
        let xpos = if applied_velocity.x > 0. { aabb.0.x + pos.x } else { pos.x }; 
        let mut stopped = false;
        for x in extent(xpos, applied_velocity.x).into_iter().skip(1) {
            let pos_x = Pos {x: x as f32, y: pos.y, z: pos.z, realm: pos.realm };
            if blocs_perp_x(pos_x, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if applied_velocity.x > 0. { 
                    pos.x = pos_x.x - aabb.0.x - 0.001;
                } else { 
                    pos.x = pos_x.x + 1.001;
                }
                velocity.0.x = 0.;
                stopped = true;
                break;
            }  
        }
        if !stopped {
            pos.x += applied_velocity.x;
        }
        // y
        let ypos: f32 = if applied_velocity.y > 0. { aabb.0.y + pos.y } else { pos.y }; 
        let mut stopped = false;
        for y in extent(ypos, applied_velocity.y).into_iter().skip(1) {
            let pos_y = Pos {x: pos.x, y: y as f32, z: pos.z, realm: pos.realm };
            if blocs_perp_y(pos_y, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if applied_velocity.y > 0. {
                    pos.y = pos_y.y - aabb.0.y - 0.001;
                } else { 
                    pos.y = pos_y.y + 1.001;
                }
                velocity.0.y = 0.;
                stopped = true;
                break;
            }
        }
        if !stopped {
            pos.y += applied_velocity.y;
        }
        // z
        let zpos: f32 = if applied_velocity.z > 0. { aabb.0.z + pos.z } else { pos.z }; 
        let mut stopped = false;
        for z in extent(zpos, applied_velocity.z).into_iter().skip(1) {
            let pos_z = Pos {x: pos.x, y: pos.y, z: z as f32, realm: pos.realm };
            if blocs_perp_z(pos_z, aabb).any(|pos| !blocs.get_block(pos).traversable()) {
                // there's a collision in this direction, stop at the block limit
                if applied_velocity.z > 0. { 
                    pos.z = pos_z.z - aabb.0.z - 0.001;
                } else { 
                    pos.z = pos_z.z + 1.001;
                }
                velocity.0.z = 0.;
                stopped = true;
                break;
            }
        }
        if !stopped {
            pos.z += applied_velocity.z;
        }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, process_jumps)
            .add_systems(Update, apply_acc)
            .add_systems(Update, apply_gravity)
            .add_systems(Update, apply_speed)
            ;
    }

}