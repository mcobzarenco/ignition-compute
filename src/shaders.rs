use bevy::{asset::load_internal_asset, prelude::*};

use crate::compute::prelude::*;

pub trait InternalComputeShader {
    fn load_shader(app: &mut App);

    fn entry_point<'a>() -> &'a str {
        "main"
    }
}

trait ComputeShaderHandle {
    fn shader_handle() -> Handle<Shader>;
}

impl<T: InternalComputeShader + TypeUuid> ComputeShaderHandle for T {
    fn shader_handle() -> Handle<Shader> {
        Handle::Weak(AssetId::Uuid {
            uuid: Self::TYPE_UUID,
        })
        .into()
    }
}

impl<T: ComputeShaderHandle + TypeUuid + Sync + Send + 'static> ComputeShader for T { 
    fn shader() -> ShaderRef {
        Self::shader_handle().into()
    }
}

// Shaders

#[derive(TypeUuid)]
#[uuid = "5747af29-0ff7-4f3a-8051-a4ce52fcb4a8"]
pub struct ParticleShader;

impl InternalComputeShader for ParticleShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/particle.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "ff5bae25-269b-4527-be6d-fe969aaca929"]
pub struct KernelShader;

impl InternalComputeShader for KernelShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/kernel.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "27a7a381-e416-49b3-9349-535b7254e0c3"]
pub struct DensityShader;

impl InternalComputeShader for DensityShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/density.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "1374bdb5-b39e-42ca-a0d9-8798a64dda1d"]
pub struct StateEquationShader;

impl InternalComputeShader for StateEquationShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/state-equation.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "4187cc3b-167d-4eb0-9995-ccbf37123f0f"]
pub struct SpatialCommonShader;

impl InternalComputeShader for SpatialCommonShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/spatial-index/common.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "3ed04e83-995a-475e-9927-9a9c99d98ebd"]
pub struct SpatialComputeEntriesShader;

impl InternalComputeShader for SpatialComputeEntriesShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/spatial-index/compute-entries.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "673bed95-edc8-456e-9327-12541dc9e56b"]
pub struct SpatialSortEntriesShader;

impl InternalComputeShader for SpatialSortEntriesShader {
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/spatial-index/sort-entries.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(TypeUuid)]
#[uuid = "0df301d1-6944-49eb-ae86-bc5bf078bd7b"]
pub struct SpatialComputeStartIndices;

impl InternalComputeShader for SpatialComputeStartIndices { 
    fn load_shader(app: &mut App) {
        load_internal_asset!(
            app,
            Self::shader_handle(),
            "sph/spatial-index/compute-start-indices.wgsl",
            Shader::from_wgsl
        );
    }
}
