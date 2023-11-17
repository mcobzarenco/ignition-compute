#define_import_path ignition::particle

struct Parameters {
    speed: f32,
    gravity_multiplier: f32,
    rest_density: f32,
    gas_constant: f32,
    viscosity: f32,
    length_scale: f32,
    particle_radius: f32,
    particle_area: f32,
    delta_time: f32,
    max_pressure_grad: f32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
}

struct Density {
    value: f32,
    number: f32,
}
