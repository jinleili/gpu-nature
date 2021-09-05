const OBSTACLE_RADIUS: f32 = 16.0;

mod lattice;
use lattice::*;

mod particle_render_node;
use particle_render_node::ParticleRenderNode;

mod d2q9_node;
use d2q9_node::D2Q9Node;
mod aa_d2q9_node;
use aa_d2q9_node::AAD2Q9Node;
mod d3q15_node;
use d3q15_node::D3Q15Node;

mod fluid_player;
pub use fluid_player::FluidPlayer;
mod d3_fluid_player;
pub use d3_fluid_player::D3FluidPlayer;
mod d3_particles_render_node;
pub use d3_particles_render_node::D3ParticleRenderNode;

use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct LbmUniform {
    pub tau: f32,
    pub omega: f32,
    // fluid type, used fot storage buffer initialization
    // 0: poiseuille, 1: custom
    pub fluid_ty: i32,
    // structure of array (put the same direction of all lattice together ) lattice data offset
    pub soa_offset: i32,
    //  D2Q9 lattice direction coordinate:
    // 6 2 5
    // 3 0 1
    // 7 4 8
    // components xy: lattice direction, z: direction's weight, z: direction's max value
    pub e_w_max: [[f32; 4]; 9],
    pub inversed_direction: [[i32; 4]; 9],
}

impl LbmUniform {
    pub fn new(tau: f32, fluid_ty: i32, soa_offset: i32) -> Self {
        LbmUniform {
            tau,
            omega: 1.0 / tau,
            fluid_ty,
            soa_offset,
            // lattice direction's weight
            e_w_max: [
                [0.0, 0.0, 0.444444, 0.6],
                [1.0, 0.0, 0.111111, 0.2222],
                [0.0, -1.0, 0.111111, 0.2222],
                [-1.0, 0.0, 0.111111, 0.2222],
                [0.0, 1.0, 0.111111, 0.2222],
                [1.0, -1.0, 0.0277777, 0.1111],
                [-1.0, -1.0, 0.0277777, 0.1111],
                [-1.0, 1.0, 0.0277777, 0.1111],
                [1.0, 1.0, 0.0277777, 0.1111],
            ],
            inversed_direction: [
                [0; 4], [3; 4], [4; 4], [1; 4], [2; 4], [7; 4], [8; 4], [5; 4], [6; 4],
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct TickTockUniforms {
    pub read_offset: [i32; 9],
    pub write_offset: [i32; 9],
}

fn is_sd_sphere(p: &idroid::math::Position, r: f32) -> bool {
    if p.length() > r {
        false
    } else {
        true
    }
}
