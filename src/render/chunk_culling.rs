use std::sync::atomic::{AtomicU32, Ordering};
use bevy::{
    camera::{primitives::{Aabb, Frustum, Sphere}, visibility::{NoFrustumCulling, Visibility}, Camera}, ecs::{query::{Changed, With}, system::Query}, math::{I64Vec3, Vec3A}, transform::components::{GlobalTransform, Transform}
};
use crate::world::chunk_pos;
use crate::block::Face;

pub fn chunk_culling(
    view_query: Query<(&Frustum, &Camera, &GlobalTransform), Changed<Frustum>>,
    mut chunk_query: Query<
        (&mut Visibility, &Transform, &Face, &Aabb),
        With<NoFrustumCulling>,
    >,
) {
    for (frustum, camera, gtransform) in view_query.iter() {
        if !camera.is_active {
            continue;
        }

        let chunk_cam_pos = chunk_pos(gtransform.translation());

        let total = AtomicU32::new(0);
        let visible = AtomicU32::new(0);

        // TODO: make it less dumb when having multiple cameras
        chunk_query.par_iter_mut().for_each(|item| {
            let (mut visibility, coord, face, aabb) = item;
            let center = Vec3A::from(coord.translation) + aabb.center;
            let world_sphere = Sphere {
                center,
                radius: aabb.half_extents.length(),
            };
            let world_aabb = Aabb {
                center,
                half_extents: aabb.half_extents,
            };
            total.fetch_add(1, Ordering::AcqRel);
            *visibility = if !face_visible(&chunk_cam_pos, chunk_pos(coord.translation), face)
                || !frustum.intersects_sphere(&world_sphere, false)
                || !intersects_aabb(frustum, &world_aabb)
            {
                Visibility::Hidden
            } else {
                visible.fetch_add(1, Ordering::AcqRel);
                Visibility::Visible
            };
        });
    }
}


fn intersects_aabb(frustum: &Frustum, aabb: &Aabb) -> bool {
    let min = aabb.min();
    let max = aabb.max();
    for half_space in &frustum.half_spaces[..5] {
        let mask: [u32; 3] = half_space.normal().cmpgt(Vec3A::ZERO).into();
        let mask = Vec3A::from_array(mask.map(|b| b as f32));
        let vmax = (mask * max + (1.0 - mask) * min).extend(1.0);
        let normal = half_space.normal_d();
        if normal.dot(vmax) < 0.0 {
            return false;
        }
    }
    true
}

fn face_visible(from: &I64Vec3, coord: I64Vec3, face: &Face) -> bool {
    let rel_coord = coord - *from;
    let res = match face {
        Face::Left => rel_coord.x >= 0,
        Face::Down => rel_coord.y >= 0,
        Face::Back => rel_coord.z >= 0,
        Face::Right => rel_coord.x <= 0,
        Face::Up => rel_coord.y <= 0,
        Face::Front => rel_coord.z <= 0,
    };
    res
}