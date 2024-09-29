#import ignition::spatial_index::common::{hash_position, SpatialIndexEntry}
#import ignition::particle::Parameters

@group(0) @binding(0) var<uniform>          params: Parameters;
@group(0) @binding(1) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, write>  entries: array<SpatialIndexEntry>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let num_particles = arrayLength(&positions);

    let particle_index = invocation_id.x;
    if (particle_index >= num_particles) {
        return;
    }

    let cell_key = get_cell_key(positions[particle_index], params.length_scale, num_particles);
    entries[particle_index] = SpatialIndexEntry(cell_key, particle_index);
}

