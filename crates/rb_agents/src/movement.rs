use bevy::{
    prelude::*,
    time::{Time, Timer},
};
use itertools::{Itertools, iproduct};
use rb_block::Block;
use rb_world::{BlockPos, Realm, VoxelWorld};
const FREE_FLY_Y_SPEED: f32 = 100.;
const ACC_MULT: f32 = 150.;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_stepped_block)
            .add_systems(Update, process_jumps)
            .add_systems(Update, process_freefly)
            .add_systems(Update, apply_acc_free_fly)
            .add_systems(Update, (apply_acc, apply_gravity, apply_velocity).chain());
    }
}

#[derive(Component)]
pub struct SteppingOn(pub Block);

#[derive(Component)]
pub struct Walking;

#[derive(Component)]
pub struct FreeFly;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Jumping {
    pub force: f32,
    pub cd: Timer,
    pub intent: bool,
}

#[derive(Component)]
pub struct Crouching(pub bool);

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
        ((v.floor() as i32)..=((size + v).floor() as i32)).collect_vec()
    } else {
        (((size + v).floor() as i32)..=(v.floor() as i32))
            .rev()
            .collect_vec()
    }
}

fn blocks_perp_to_axis(
    pos: Vec3,
    realm: Realm,
    aabb: &AABB,
    normal_axis: usize,
    plane_axis1: usize,
    plane_axis2: usize,
) -> impl Iterator<Item = BlockPos> {
    let normal_pos = pos[normal_axis].floor() as i32;
    iproduct!(
        extent(pos[plane_axis1], aabb.0[plane_axis1]),
        extent(pos[plane_axis2], aabb.0[plane_axis2])
    )
    .map(move |(pos1, pos2)| {
        let mut pos = BlockPos {
            realm: realm,
            ..Default::default()
        };
        pos[normal_axis] = normal_pos;
        pos[plane_axis1] = pos1;
        pos[plane_axis2] = pos2;
        pos
    })
}

fn update_stepped_block(
    blocks: Res<VoxelWorld>,
    mut query: Query<(&Transform, &Realm, &AABB, &mut SteppingOn)>,
) {
    for (transform, realm, aabb, mut stepping_on) in query.iter_mut() {
        let below = transform.translation + Vec3::new(0., -0.01, 0.);
        let mut closest_block = Block::Air;
        let mut min_dist = f32::INFINITY;
        for block_pos in blocks_perp_to_axis(below, *realm, aabb, 1, 0, 2) {
            let block = blocks.get_block(block_pos);
            if block.is_traversable() {
                continue;
            }
            let dist = (below.x - block_pos.x as f32).abs() - (below.y - block_pos.y as f32).abs();
            if dist < min_dist {
                min_dist = dist;
                closest_block = block;
            }
        }
        stepping_on.0 = closest_block;
    }
}

fn process_jumps(
    time: Res<Time>,
    mut query: Query<(&mut Jumping, &mut Velocity, &SteppingOn), With<Walking>>,
) {
    for (mut jumping, mut velocity, stepping_on) in query.iter_mut() {
        jumping.cd.tick(time.delta());
        if jumping.intent && jumping.cd.is_finished() && !stepping_on.0.is_traversable() {
            velocity.0.y += jumping.force;
            jumping.cd.reset();
        }
    }
}

fn process_freefly(mut query: Query<(&mut Velocity, &Jumping, &Crouching), With<FreeFly>>) {
    for (mut velocity, jumping, crouching) in query.iter_mut() {
        velocity.0.y = (jumping.intent as i32 - crouching.0 as i32) as f32 * FREE_FLY_Y_SPEED;
    }
}

fn apply_gravity(
    blocks: Res<VoxelWorld>,
    time: Res<Time>,
    mut query: Query<(&Transform, &Realm, &mut Velocity, &Gravity), With<Walking>>,
) {
    for (transform, realm, mut velocity, gravity) in query.iter_mut() {
        if !blocks.is_col_loaded(transform.translation, *realm) {
            continue;
        }
        velocity.0 += Vec3::new(0., -gravity.0 * time.delta_secs(), 0.);
    }
}

fn apply_acc(
    blocks: Res<VoxelWorld>,
    time: Res<Time>,
    mut query: Query<(&Heading, &mut Velocity, &Transform, &Realm, &SteppingOn), With<Walking>>,
) {
    for (heading, mut velocity, transform, realm, stepping_on) in query.iter_mut() {
        if !blocks.is_col_loaded(transform.translation, *realm) {
            continue;
        }
        // get the block the entity is standing on if the entity has an AABB
        let friction: f32 = stepping_on.0.friction();
        let slowing: f32 = stepping_on.0.slowing();
        // applying slowing
        let heading = heading.0 * slowing;
        // make velocity inch towards heading
        let mut diff = heading - velocity.0;
        if diff.y.is_nan() {
            diff.y = 0.;
        }
        let diff_len = diff.length();
        if diff_len == 0. {
            continue;
        }
        let c = (time.delta_secs() * friction * ACC_MULT / diff_len.max(1.)).min(1.);
        let acc: Vec3 = c * diff;
        velocity.0 += acc;
    }
}

fn apply_acc_free_fly(mut query: Query<(&Heading, &mut Velocity), With<FreeFly>>) {
    for (heading, mut velocity) in query.iter_mut() {
        velocity.0.x = heading.0.x;
        velocity.0.z = heading.0.z;
    }
}

fn apply_velocity(
    blocks: Res<VoxelWorld>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, &Realm, &AABB)>,
) {
    for (mut velocity, mut transform, realm, aabb) in query.iter_mut() {
        if !blocks.is_col_loaded(transform.translation, *realm) {
            continue;
        }
        let applied_velocity = velocity.0 * time.delta_secs();
        // split the motion on all 3 axis, check for collisions, adjust the final speed vector if there's any
        for (normal, axis1, axis2) in [(0, 1, 2), (1, 0, 2), (2, 0, 1)] {
            let pos = transform.translation[normal]
                + if applied_velocity[normal] > 0. {
                    aabb.0[normal]
                } else {
                    0.
                };
            let mut stopped = false;
            for i in extent(pos, applied_velocity[normal]).into_iter().skip(1) {
                let mut pos = transform.translation;
                pos[normal] = i as f32;
                if blocks_perp_to_axis(pos, *realm, aabb, normal, axis1, axis2)
                    .any(|pos| !blocks.get_block(pos).is_traversable())
                {
                    // there's a collision in this direction, stop at the block limit
                    if applied_velocity[normal] > 0. {
                        transform.translation[normal] = pos[normal] - aabb.0[normal] - 0.001;
                    } else {
                        transform.translation[normal] = pos[normal] + 1.001;
                    }
                    velocity.0[normal] = 0.;
                    stopped = true;
                    break;
                }
            }
            if !stopped {
                transform.translation[normal] += applied_velocity[normal];
            }
        }
    }
}
