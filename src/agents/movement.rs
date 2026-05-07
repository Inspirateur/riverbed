use crate::Block;
use crate::world::{BlockPos, Realm, VoxelWorld};
use avian3d::prelude::{Friction, LinearVelocity, LockedAxes, ShapeHits};
use bevy::{
    prelude::*,
    time::{Time, Timer},
};

const FREE_FLY_Y_SPEED: f32 = 100.;
const ACC_MULT: f32 = 150.;
// Pulls voxel-intersection probes inward so resting contact (Avian sinks the
// collider into the floor by ~0.005-0.02 blocks) doesn't false-trigger freeze.
const VOXEL_INTERSECT_INSET: f32 = 0.05;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_grounded_and_material)
            .add_systems(PreUpdate, freeze_when_voxel_intersected)
            .add_systems(Update, process_jumps)
            .add_systems(Update, process_freefly)
            .add_systems(Update, apply_acc_free_fly)
            .add_systems(Update, apply_acc);
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

/// Half-extents of the entity's body, in body-local coordinates. Only `y` is
/// read (sets the vertical span of `freeze_when_voxel_intersected`'s probes).
#[derive(Component)]
pub struct BodyExtents(pub Vec3);

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Heading(pub Vec3);

fn update_grounded_and_material(
    blocks: Res<VoxelWorld>,
    mut query: Query<(
        &Realm,
        &ShapeHits,
        &mut Grounded,
        &mut SteppingOn,
        &mut Friction,
    )>,
) {
    for (realm, hits, mut grounded, mut stepping_on, mut friction) in query.iter_mut() {
        if let Some(hit) = hits.iter().next() {
            grounded.0 = true;
            // Sample just below the hit surface so we identify the block under
            // the contact, not the air immediately above it.
            let p = hit.point1 - Vec3::Y * 0.05;
            stepping_on.0 = blocks.get_block(BlockPos {
                x: p.x.floor() as i32,
                y: p.y.floor() as i32,
                z: p.z.floor() as i32,
                realm: *realm,
            });
        } else {
            grounded.0 = false;
            stepping_on.0 = Block::Air;
        }
        let f = stepping_on.0.friction();
        friction.dynamic_coefficient = f;
        friction.static_coefficient = f;
    }
}

/// Locks all axes when any of three axis-line probes is inside a non-traversable
/// voxel. Three probes (top, mid, bottom) are sufficient because their 0.80m
/// spacing is less than block height, so any block intersecting the body's
/// vertical span contains at least one. Tests the central axis rather than the
/// AABB so the rounded sides of the capsule don't false-trigger freeze when they
/// brush a corner block the capsule isn't actually touching.
fn freeze_when_voxel_intersected(
    blocks: Res<VoxelWorld>,
    mut query: Query<(
        &Transform,
        &Realm,
        &BodyExtents,
        &mut LockedAxes,
        &mut LinearVelocity,
    )>,
) {
    for (transform, realm, extents, mut locked, mut velocity) in query.iter_mut() {
        let c = transform.translation;
        let inset_h = extents.0.y - VOXEL_INTERSECT_INSET;
        let probes = [
            c + Vec3::Y * inset_h,
            c,
            c - Vec3::Y * inset_h,
        ];
        let intersecting = probes.iter().any(|p| {
            !blocks
                .get_block(BlockPos {
                    x: p.x.floor() as i32,
                    y: p.y.floor() as i32,
                    z: p.z.floor() as i32,
                    realm: *realm,
                })
                .is_traversable()
        });
        if intersecting {
            *locked = LockedAxes::ALL_LOCKED;
            velocity.0 = Vec3::ZERO;
        } else {
            *locked = LockedAxes::ROTATION_LOCKED;
        }
    }
}

fn process_jumps(
    time: Res<Time>,
    mut query: Query<(&mut Jumping, &mut LinearVelocity, &Grounded), With<Walking>>,
) {
    for (mut jumping, mut velocity, grounded) in query.iter_mut() {
        jumping.cd.tick(time.delta());
        if jumping.intent && jumping.cd.is_finished() && grounded.0 {
            velocity.0.y = jumping.force;
            jumping.cd.reset();
        }
    }
}

fn process_freefly(
    mut query: Query<(&mut LinearVelocity, &Jumping, &Crouching), With<FreeFly>>,
) {
    for (mut velocity, jumping, crouching) in query.iter_mut() {
        velocity.0.y = (jumping.intent as i32 - crouching.0 as i32) as f32 * FREE_FLY_Y_SPEED;
    }
}

fn apply_acc(
    blocks: Res<VoxelWorld>,
    time: Res<Time>,
    mut query: Query<
        (&Heading, &mut LinearVelocity, &Transform, &Realm, &SteppingOn),
        With<Walking>,
    >,
) {
    for (heading, mut velocity, transform, realm, stepping_on) in query.iter_mut() {
        if !blocks.is_col_loaded(transform.translation, *realm) {
            continue;
        }
        let target = Vec2::new(heading.0.x, heading.0.z) * stepping_on.0.slowing();
        let current = Vec2::new(velocity.0.x, velocity.0.z);
        let diff = target - current;
        let diff_len = diff.length();
        if diff_len == 0. {
            continue;
        }
        let c = (time.delta_secs() * ACC_MULT / diff_len.max(1.)).min(1.);
        let acc = c * diff;
        velocity.0.x += acc.x;
        velocity.0.z += acc.y;
    }
}

fn apply_acc_free_fly(mut query: Query<(&Heading, &mut LinearVelocity), With<FreeFly>>) {
    for (heading, mut velocity) in query.iter_mut() {
        velocity.0.x = heading.0.x;
        velocity.0.z = heading.0.z;
    }
}
