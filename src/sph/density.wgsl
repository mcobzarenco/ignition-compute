#import ignition::kernel::cubic_spline_kernel
#import ignition::particle::{Density, Parameters, Particle}

@group(0) @binding(0) var<uniform>              params: Parameters;
@group(0) @binding(1) var<storage, read> particles_src: array<Particle>;
@group(0) @binding(2) var<storage, read_write> density: array<Density>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let num_particles = arrayLength(&particles_src); 

    let i = invocation_id.x;
    if (i >= num_particles) {
        return;
    }

    var number_density_i = 0.0;
    var velocity_i = particles_src[i].velocity + vec2<f32>(0.0, -params.gravity_multiplier) * params.delta_time;
    var position_i = particles_src[i].position + velocity_i * params.delta_time;
    // let position_i = particles_src[i].position;

    var j: u32 = 0u;
    loop {
        if (j >= num_particles) {
            break;
        }
        let velocity_j = particles_src[j].velocity + vec2<f32>(0.0, -params.gravity_multiplier) * params.delta_time;
        let position_j = particles_src[j].position + velocity_j * params.delta_time;
        // let position_j = particles_src[j].position;
	let x_ij = position_i - position_j;
	
	number_density_i += cubic_spline_kernel(x_ij, params.length_scale);

        continuing {
            j = j + 1u;
        }
    }

    density[i].value = number_density_i;
    density[i].number = number_density_i;
}

