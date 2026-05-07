use bevy::math::Vec3;
use crate::block::Face;
// Note: This whole file will become unnecessary when we have instancing,
// because it is used to convert a quad to 4 vertices which we won't need to do

fn packed_xyz(x: u32, y: u32, z: u32) -> u32 {
    (z << 12) | (y << 6) | x
}

fn vertex_info(xyz: u32, u: u32, v: u32) -> u32 {
    (v << 24) | (u << 18) | xyz
}

impl Face {
    pub fn vertices_packed(&self, xyz: u32, w: u32, h: u32, lod: u32) -> [u32; 4] {
        let xyz = xyz*lod;
        let w_ = w*lod;
        let h_ = h*lod;
        match self {
            Face::Left => [
                vertex_info(xyz, h, w),
                vertex_info(xyz+packed_xyz(0, 0, h_), 0, w),
                vertex_info(xyz+packed_xyz(0, w_, 0), h, 0),
                vertex_info(xyz+packed_xyz(0, w_, h_), 0, 0),
            ],
            Face::Down => [
                vertex_info(xyz-packed_xyz(w_, 0, 0)+packed_xyz(0, 0, h_), w, h),
                vertex_info(xyz-packed_xyz(w_, 0, 0), w, 0),
                vertex_info(xyz+packed_xyz(0, 0, h_), 0, h),
                vertex_info(xyz, 0, 0),
            ],
            Face::Back => [
                vertex_info(xyz, w, h),
                vertex_info(xyz+packed_xyz(0, h_, 0), w, 0),
                vertex_info(xyz+packed_xyz(w_, 0, 0), 0, h),
                vertex_info(xyz+packed_xyz(w_, h_, 0), 0, 0),
            ],
            Face::Right => [
                vertex_info(xyz, 0, 0),
                vertex_info(xyz+packed_xyz(0, 0, h_), h, 0),
                vertex_info(xyz-packed_xyz(0, w_, 0), 0, w),
                vertex_info(xyz+packed_xyz(0, 0, h_)-packed_xyz(0, w_, 0), h, w),
            ],
            Face::Up => [
                vertex_info(xyz+packed_xyz(w_, 0, h_), w, h),
                vertex_info(xyz+packed_xyz(w_, 0, 0), w, 0),
                vertex_info(xyz+packed_xyz(0, 0, h_), 0, h),
                vertex_info(xyz, 0, 0),
            ],
            Face::Front => [
                vertex_info(xyz-packed_xyz(w_, 0, 0)+packed_xyz(0, h_, 0), 0, 0),
                vertex_info(xyz-packed_xyz(w_, 0, 0), 0, h),
                vertex_info(xyz+packed_xyz(0, h_, 0), w, 0),
                vertex_info(xyz, w, h),
            ],
        }
    }

    /// `Vec3` companion to `vertices_packed` for physics collider geometry.
    /// Same per-face corner ordering, so `bgm::indices`' winding still applies.
    pub fn vertices_f32(&self, x: u32, y: u32, z: u32, w: u32, h: u32, lod: u32) -> [Vec3; 4] {
        let lod = lod as f32;
        let x = x as f32 * lod;
        let y = y as f32 * lod;
        let z = z as f32 * lod;
        let w = w as f32 * lod;
        let h = h as f32 * lod;
        let p = |dx: f32, dy: f32, dz: f32| Vec3::new(x + dx, y + dy, z + dz);
        match self {
            Face::Left => [
                p(0., 0., 0.),
                p(0., 0., h),
                p(0., w, 0.),
                p(0., w, h),
            ],
            Face::Down => [
                p(-w, 0., h),
                p(-w, 0., 0.),
                p(0., 0., h),
                p(0., 0., 0.),
            ],
            Face::Back => [
                p(0., 0., 0.),
                p(0., h, 0.),
                p(w, 0., 0.),
                p(w, h, 0.),
            ],
            Face::Right => [
                p(0., 0., 0.),
                p(0., 0., h),
                p(0., -w, 0.),
                p(0., -w, h),
            ],
            Face::Up => [
                p(w, 0., h),
                p(w, 0., 0.),
                p(0., 0., h),
                p(0., 0., 0.),
            ],
            Face::Front => [
                p(-w, h, 0.),
                p(-w, 0., 0.),
                p(0., h, 0.),
                p(0., 0., 0.),
            ],
        }
    }
}