use crate::blocks::Face;
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
}