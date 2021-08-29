use super::{is_sd_sphere, OBSTACLE_RADIUS};
use crate::FieldAnimationType;
use idroid::math::Position;
use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct LatticeInfo {
    pub material: i32,
    //  dynamic iter value, change material ultimately
    pub block_iter: i32,
    pub vx: f32,
    pub vy: f32,
}

pub enum LatticeType {
    BULK = 1,
    BOUNDARY = 2,
    INLET = 3,
    OBSTACLE = 4,
    OUTLET = 5,
}

pub fn init_lattice_material(
    lattice_size: wgpu::Extent3d, ty: FieldAnimationType,
) -> Vec<LatticeInfo> {
    let mut info: Vec<LatticeInfo> = vec![];
    let (nx, ny, nz) =
        (lattice_size.width, lattice_size.height, lattice_size.depth_or_array_layers);
    let s0 = Position::new(nx as f32 / 7.0 - OBSTACLE_RADIUS, ny as f32 / 2.0);
    let s1 = Position::new(nx as f32 / 5.0, ny as f32 / 3.0);
    let s2 = Position::new(nx as f32 / 5.0, ny as f32 * 0.66);
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let mut material = LatticeType::BULK as i32;
                let mut vx = 0.0;

                // need boundary cell to avoid NAN
                match ty {
                    FieldAnimationType::Custom => {
                        if x == 0 || x == nx - 1 || y == 0 || y == ny - 1 {
                            material = LatticeType::BOUNDARY as i32;
                        }
                    }
                    FieldAnimationType::Poiseuille => {
                        // poiseuille
                        if y == 0 || y == ny - 1 || (nz > 1 && (z == 0 || z == nz - 1)) {
                            material = LatticeType::BOUNDARY as i32;
                        } else if x == 0 {
                            material = LatticeType::INLET as i32;
                            vx = 0.12;
                        } else if x == nx - 1 {
                            material = LatticeType::OUTLET as i32;
                        } else {
                            // obstacle
                            let p = Position::new(x as f32, y as f32);
                            if is_sd_sphere(&p.minus(&s0), OBSTACLE_RADIUS)
                                || is_sd_sphere(&p.minus(&s1), OBSTACLE_RADIUS)
                                || is_sd_sphere(&p.minus(&s2), OBSTACLE_RADIUS)
                            {
                                material = LatticeType::OBSTACLE as i32;
                            }
                        }
                    }
                    _ => {}
                }

                info.push(LatticeInfo { material, block_iter: -1, vx, vy: 0.0 });
            }
        }
    }

    info
}
