#import ignition::kernel::spiky_kernel_grad
#import ignition::particle::{Density, Parameters, Particle}

@group(0) @binding(0) var<uniform>                    params: Parameters;
@group(0) @binding(1) var<storage, read>       particles_src: array<Particle>;
@group(0) @binding(2) var<storage, read>             density: array<Density>;
@group(0) @binding(3) var<storage, read_write> particles_dst: array<Particle>;

const GAS_CONSTANT: f32 = 1000.0; 
const REST_DENSITY: f32 = 16.0; 
const GAMMA: f32 = 3.0; // "adiabatic index"

@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let num_particles = arrayLength(&particles_src);

    let i = invocation_id.x;
    if (i >= num_particles) {
        return;
    }

    var velocity_i = particles_src[i].velocity + vec2<f32>(0.0, -params.gravity_multiplier) * params.delta_time;
    var position_i = particles_src[i].position + velocity_i * params.delta_time;

    // var velocity_i = particles_src[i].velocity;
    // var position_i = particles_src[i].position;

    let density_i = density[i];
    var pressure_i = vec2<f32>(0.0, 0.0);

    // let point_pressure_i = GAS_CONSTANT * (density_i.value / REST_DENSITY - 1.0);
    let point_pressure_i = GAS_CONSTANT * REST_DENSITY / GAMMA * (pow(density_i.value / REST_DENSITY, GAMMA) - 1.0); 
    let normalized_point_pressure_i = point_pressure_i / (density_i.number * density_i.number + 1e-4);

    var j: u32 = 0u;
    loop {
        if (j >= num_particles) {
            break;
        }

        if (j == i) {
            continue;
        }

        let velocity_j = particles_src[j].velocity + vec2<f32>(0.0, -params.gravity_multiplier) * params.delta_time;
        let position_j = particles_src[j].position + velocity_j * params.delta_time;
	let density_j = density[j];

	let x_ij = position_i - position_j;
	let grad_w_ij = -spiky_kernel_grad(x_ij, params.length_scale);

	// let point_pressure_j = GAS_CONSTANT * (density_j.value / REST_DENSITY - 1.0);
	let point_pressure_j = GAS_CONSTANT * REST_DENSITY / GAMMA * (pow(density_j.value / REST_DENSITY, GAMMA) - 1.0); 
	let normalized_point_pressure_j = point_pressure_j / (density_j.number * density_j.number + 1e-4);

	pressure_i = pressure_i + (normalized_point_pressure_i + normalized_point_pressure_j) * grad_w_ij;
   
        continuing {
            j = j + 1u;
        }
    }

    pressure_i = normalize(pressure_i) * clamp(length(pressure_i), 0.0, 1000.0);

    // pressure_i.grad +=
    //   (normalized_point_pressure_i + normalized_point_pressure_j) * grad_w_ij;
    // pressure_i.grad = pressure_i.grad.clamp_length_max(params.max_pressure_grad);
    
    // kinematic update
    velocity_i = velocity_i + pressure_i * params.delta_time;
    position_i = particles_src[i].position + velocity_i * params.delta_time; 

    // velocity_i = velocity_i + vec2<f32>(0.0, -params.gravity_multiplier) * params.speed;
    // clamp velocity for a more pleasing simulation
    // velocity_i = normalize(velocity_i) * clamp(length(velocity_i), 0.0, 100.0 * params.speed); 
    
    // Wrap around boundary
    if (position_i.x < params.x_min) {
        position_i.x = params.x_min + (params.x_min - position_i.x);
	velocity_i.x = -velocity_i.x * 0.9;
    }
    if (position_i.x > params.x_max) {
        position_i.x = params.x_max + (params.x_max - position_i.x);
	velocity_i.x = -velocity_i.x * 0.9;
    }
    if (position_i.y < params.y_min) {
        position_i.y = params.y_min + (params.y_min - position_i.y);
	velocity_i.y = -velocity_i.y * 0.9;
    }
    if (position_i.y > params.y_max) {
        position_i.y = params.y_max + (params.y_max - position_i.y);
	velocity_i.y = -velocity_i.y * 0.9;
    }
 
    // Write back
    particles_dst[i].position = position_i;
    particles_dst[i].velocity = velocity_i;
}

