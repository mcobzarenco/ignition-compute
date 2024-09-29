use bevy::{core::Pod, prelude::*, render::render_resource::ShaderType};
use bytemuck::Zeroable;

#[derive(Resource, Reflect)]
pub struct Positions(Vec<Vec2>);

#[derive(Resource, Reflect)]
pub struct Velocities(Vec<Vec2>);

#[derive(ShaderType, Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
struct Density {
    value: f32,
    number: f32,
}
