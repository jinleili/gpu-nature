use std::{borrow::BorrowMut, u32};

use super::{init_lattice_material, is_sd_sphere, LatticeInfo, LatticeType, OBSTACLE_RADIUS};
use crate::{
    create_shader_module, fluid::LbmUniform, setting_obj::SettingObj, FieldAnimationType,
    FieldUniform,
};
use idroid::{
    math::{Position, Size},
    node::{BindingGroupSettingNode, ComputeNode},
    AnyTexture, BufferObj,
};
use wgpu::{TextureFormat, TextureView};
use zerocopy::AsBytes;

pub struct D2Q9Node {
    pub lattice: wgpu::Extent3d,
    pub lattice_pixel_size: u32,
    animation_ty: FieldAnimationType,
    pub lbm_uniform_buf: BufferObj,
    pub fluid_uniform_buf: BufferObj,
    pub macro_tex: AnyTexture,
    pub lattice_info_data: Vec<LatticeInfo>,
    pub info_buf: BufferObj,
    setting_nodes: Vec<BindingGroupSettingNode>,
    collide_stream_pipelines: Vec<wgpu::ComputePipeline>,
    boundary_pipelines: Vec<wgpu::ComputePipeline>,
    pub threadgroup_count: (u32, u32, u32),
    pub reset_node: ComputeNode,
}

impl D2Q9Node {
    pub fn new(
        device: &wgpu::Device, queue: &wgpu::Queue, canvas_size: Size<u32>, setting: &SettingObj,
    ) -> Self {
        let lattice_pixel_size = 4;
        let lattice = wgpu::Extent3d {
            width: canvas_size.width / lattice_pixel_size,
            height: canvas_size.height / lattice_pixel_size,
            depth_or_array_layers: 1,
        };

        let threadgroup_count = ((lattice.width + 63) / 64, (lattice.height + 3) / 4, 1);
        println!("threadgroup_count: {:?}", threadgroup_count);
        // reynolds number: (length)(velocity)/(viscosity)
        // Kármán vortex street： 47 < Re < 10^5
        // let viscocity = (lattice.width as f32 * 0.05) / 320.0;
        // println!("-- {} --", viscocity);
        // 通过外部参数来重置流体粒子碰撞松解时间 tau = (3.0 * x + 0.5), x：[0~1] 趋大，松解时间趋快
        let tau = 3.0 * setting.fluid_viscosity + 0.5;
        // let tau = 3.0 * viscocity + 0.5;

        let fluid_ty = if setting.animation_type == FieldAnimationType::Poiseuille { 0 } else { 1 };
        let lbm_uniform_data =
            LbmUniform::new(tau, fluid_ty, (lattice.width * lattice.height) as i32);

        let (_, sx, sy) = idroid::utils::matrix_helper::fullscreen_factor(
            (canvas_size.width as f32, canvas_size.height as f32).into(),
        );
        let field_uniform_data = FieldUniform {
            lattice_size: [lattice.width as i32, lattice.height as i32, 1],
            lattice_pixel_size: [lattice_pixel_size as f32, lattice_pixel_size as f32, 1.0],
            canvas_size: [canvas_size.width as i32, canvas_size.height as i32, 0],
            normalized_space_size: [sx, sy, 0.0],
            pixel_distance: [0.0; 3],
            speed_ty: 1,
            _padding: [0; 5],
        };
        let lbm_uniform_buf =
            BufferObj::create_uniform_buffer(device, &lbm_uniform_data, Some("uniform_buf0"));
        let fluid_uniform_buf = BufferObj::create_uniform_buffer(
            device,
            &field_uniform_data,
            Some("fluid_uniform_buf"),
        );
        let scalar_lattice_size = (lattice.width * lattice.height * 4) as wgpu::BufferAddress;
        // let macro_buf = BufferObj::create_empty_storage_buffer(
        //     device,
        //     scalar_lattice_size * 4,
        //     false,
        //     Some("macro_buffer"),
        // );
        let macro_tex_format = TextureFormat::Rgba16Float;
        let macro_tex_access = wgpu::StorageTextureAccess::WriteOnly;
        let macro_tex = idroid::load_texture::empty(
            device,
            macro_tex_format,
            wgpu::Extent3d {
                width: lattice.width,
                height: lattice.height,
                depth_or_array_layers: 1,
            },
            None,
            Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING),
        );

        let lattice_info_data = init_lattice_material(lattice, setting.animation_type);
        let info_buf =
            BufferObj::create_storage_buffer(device, &lattice_info_data, Some("info_buffer"));

        let mut collide_stream_buffers: Vec<BufferObj> = vec![];
        for _ in 0..2 {
            collide_stream_buffers.push(BufferObj::create_empty_storage_buffer(
                device,
                scalar_lattice_size * 9,
                false,
                Some("lattice_buf"),
            ));
        }
        let collide_stream_shader =
            create_shader_module(device, "lbm/collide_stream", Some("collide_stream_shader"));
        let boundary_shader = create_shader_module(device, "lbm/boundary", Some("boundary_shader"));

        let visibilitys: Vec<wgpu::ShaderStages> = [wgpu::ShaderStages::COMPUTE; 10].to_vec();
        let mut setting_nodes = Vec::<BindingGroupSettingNode>::with_capacity(2);
        let mut collide_stream_pipelines = Vec::<wgpu::ComputePipeline>::with_capacity(2);
        let mut boundary_pipelines = Vec::<wgpu::ComputePipeline>::with_capacity(2);

        for i in 0..2 {
            collide_stream_buffers[i].borrow_mut().read_only = true;
            collide_stream_buffers[(i + 1) % 2].borrow_mut().read_only = false;
            let buffers =
                vec![&collide_stream_buffers[i], &collide_stream_buffers[(i + 1) % 2], &info_buf];
            let setting_node = BindingGroupSettingNode::new(
                device,
                vec![&lbm_uniform_buf, &fluid_uniform_buf],
                buffers.clone(),
                vec![(&macro_tex, Some(macro_tex_access))],
                vec![],
                visibilitys.clone(),
            );
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&setting_node.bind_group_layout],
                push_constant_ranges: &[],
            });
            let collide_stream_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("collid_stream pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &collide_stream_shader,
                    entry_point: "main",
                });
            let boundary_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("boundary_pipeline pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &boundary_shader,
                    entry_point: "main",
                });
            setting_nodes.push(setting_node);
            collide_stream_pipelines.push(collide_stream_pipeline);
            boundary_pipelines.push(boundary_pipeline);
        }

        let init_shader = create_shader_module(device, "lbm/init", Some("init_shader"));
        collide_stream_buffers[0].borrow_mut().read_only = false;
        collide_stream_buffers[1].borrow_mut().read_only = false;
        let reset_node = ComputeNode::new(
            device,
            threadgroup_count,
            vec![&lbm_uniform_buf, &fluid_uniform_buf],
            vec![&collide_stream_buffers[0], &collide_stream_buffers[1], &info_buf],
            vec![(&macro_tex, Some(macro_tex_access))],
            &init_shader,
        );

        let mut instance = D2Q9Node {
            lattice,
            lattice_pixel_size,
            animation_ty: setting.animation_type,
            lbm_uniform_buf,
            fluid_uniform_buf,
            macro_tex,
            lattice_info_data,
            info_buf,
            setting_nodes,
            threadgroup_count,
            collide_stream_pipelines,
            boundary_pipelines,
            reset_node,
        };
        // On latast wgpu(2021/06/05), must reset twice to get correct result
        // But, cannot use official demo reproduce this problem, so strange!!
        instance.reset_lattice_info(device, queue);
        instance.reset_lattice_info(device, queue);

        return instance;
    }

    pub fn reset(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.reset_node.compute(encoder);
    }

    pub fn add_obstacle(&mut self, queue: &wgpu::Queue, x: u32, y: u32) {
        let obstacle = LatticeInfo {
            material: LatticeType::OBSTACLE as i32,
            block_iter: -1,
            vx: 0.0,
            vy: 0.0,
        };
        let center = Position::new(x as f32 + 0.5, y as f32 + 0.5);
        let mut info: Vec<LatticeInfo> = vec![];

        let min_y = y - OBSTACLE_RADIUS as u32;
        let max_y = min_y + OBSTACLE_RADIUS as u32 * 2;
        for y in min_y..max_y {
            for x in 0..self.lattice.width {
                let index = (self.lattice.width * y) + x;
                if is_sd_sphere(
                    &Position::new(x as f32 + 0.5, y as f32 + 0.5).minus(&center),
                    OBSTACLE_RADIUS,
                ) {
                    self.lattice_info_data[index as usize] = obstacle;
                    info.push(obstacle);
                } else {
                    let origin_info = self.lattice_info_data[index as usize];
                    info.push(origin_info);
                }
            }
        }

        let offset = (self.lattice.width * min_y) as u64 * 16;
        queue.write_buffer(&self.info_buf.buffer, offset, info.as_bytes());
    }

    pub fn reset_lattice_info(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.animation_ty == FieldAnimationType::Poiseuille {
            self.lattice_info_data = init_lattice_material(self.lattice, self.animation_ty);
            queue.write_buffer(&self.info_buf.buffer, 0, self.lattice_info_data.as_bytes());
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid reset encoder"),
        });
        self.reset(&mut encoder);
        queue.submit(Some(encoder.finish()));
    }

    pub fn add_external_force(&mut self, queue: &wgpu::Queue, pos: Position, pre_pos: Position) {
        let dis = pos.distance(&pre_pos);
        let mut force = 0.1 * (dis / 20.0);
        if force > 0.12 {
            force = 0.12;
        }
        let ridian = pos.slope_ridian(&pre_pos);
        let vx: f32 = force * ridian.cos();
        let vy = force * ridian.sin();
        let info: Vec<LatticeInfo> =
            vec![LatticeInfo { material: LatticeType::INLET as i32, block_iter: 30, vx, vy }];
        let c = (dis / (self.lattice_pixel_size - 1) as f32).ceil();
        let step = dis / c;
        for i in 0..c as i32 {
            let p = pre_pos.new_by_slope_n_dis(ridian, step * i as f32).round();
            let x = p.x as u32 / self.lattice_pixel_size;
            let y = p.y as u32 / self.lattice_pixel_size;
            if x < 1 || x >= self.lattice.width - 2 || y < 1 || y >= self.lattice.height - 2 {
                continue;
            }
            let offset = (self.lattice.width * y + x) as u64 * 16;
            queue.write_buffer(&self.info_buf.buffer, offset, info.as_bytes());
        }
    }

    pub fn dispatch<'c, 'b: 'c>(&'b self, cpass: &mut wgpu::ComputePass<'c>, swap_index: usize) {
        cpass.set_bind_group(0, &self.setting_nodes[swap_index].bind_group, &[]);
        cpass.set_pipeline(&self.collide_stream_pipelines[swap_index]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);
        cpass.set_pipeline(&self.boundary_pipelines[swap_index]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);
    }
}
