mod camera;
mod compute;
mod particle;
mod shaders;
mod spatial_index;

use bevy::{
    core::Pod,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::{PresentMode, WindowPlugin},
};
use bytemuck::Zeroable;
use rand::distributions::{Distribution, Uniform};
use shaders::InternalComputeShader;

use crate::{
    camera::{PanCam, PanCamPlugin},
    compute::prelude::*,
};

const NUM_PARTICLES: u32 = 10000;

#[derive(Resource, ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
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

impl Default for Parameters {
    fn default() -> Self {
        Self {
            speed: 1.0,
            gravity_multiplier: 98.0,
            rest_density: 1.0,
            gas_constant: 2.0e9,
            viscosity: 0.08,
            length_scale: 1.0,
            particle_radius: 0.1,
            particle_area: 1.0,
            delta_time: 0.001,
            max_pressure_grad: 2000.0,
            x_min: -10.0,
            x_max: 80.0,
            y_min: 0.0,
            y_max: 10.0,
        }
    }
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
struct Density {
    value: f32,
    number: f32,
}

struct BoidWorker;

impl ComputeWorker for BoidWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = world.get_resource::<Parameters>().unwrap().clone();

        let mut initial_boids_data = Vec::with_capacity(NUM_PARTICLES as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1., 1.);

        for i in 0..NUM_PARTICLES {
            initial_boids_data.push(Particle {
                position: Vec2::new(
                    (i % 80) as f32 * (params.particle_radius * 3.0) + 1.0 + params.x_min,
                    (i / 100) as f32 * (params.particle_radius * 3.0) + 1.0 + params.y_min,
                ),
                velocity: 10.0
                    * Vec2::new(
                        unif.sample(&mut rng) * params.speed,
                        unif.sample(&mut rng) * params.speed,
                    ),
            });
        }

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_staging("particles_src", &initial_boids_data)
            .add_staging("particles_dst", &initial_boids_data)
            .add_staging(
                "density",
                &vec![
                    Density {
                        value: 0.0,
                        number: 0.0
                    };
                    NUM_PARTICLES as usize
                ],
            )
            .add_pass::<shaders::DensityShader>(
                [NUM_PARTICLES / 32 + 1, 1, 1],
                &["params", "particles_src", "density"],
            )
            .add_pass::<shaders::StateEquationShader>(
                [NUM_PARTICLES / 32 + 1, 1, 1],
                &["params", "particles_src", "density", "particles_dst"],
            )
            .add_swap("particles_src", "particles_dst")
            .build()
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1200., 1800.).into(),
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PanCamPlugin)
    .add_plugins(LogDiagnosticsPlugin::default())
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .insert_resource(ClearColor(Color::DARK_GRAY))
    .insert_resource(Parameters::default())
    .add_plugins(AppComputePlugin)
    .add_plugins(AppComputeWorkerPlugin::<BoidWorker>::default())
    .add_systems(Startup, setup)
    .add_systems(Update, move_entities);

    // load_shaders(&mut app);

    shaders::ParticleShader::load_shader(&mut app);
    shaders::KernelShader::load_shader(&mut app);
    shaders::DensityShader::load_shader(&mut app);
    shaders::StateEquationShader::load_shader(&mut app);

    app.run();
}

#[derive(Component)]
struct BoidEntity(pub usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    params: Res<Parameters>,
) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scale: 0.01,
                ..default()
            },
            ..default()
        },
        PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.001, // prevent the camera from zooming too far in
            max_scale: Some(100.0), // prevent the camera from zooming too far out
            ..default()
        },
    ));

    let boid_mesh = meshes.add(shape::Circle::new(params.particle_radius).into());
    let boid_material = materials.add(Color::ANTIQUE_WHITE.into());

    // First boid in red, so we can follow it easily
    commands.spawn((
        BoidEntity(0),
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(boid_mesh.clone()),
            material: materials.add(Color::ORANGE_RED.into()),
            ..Default::default()
        },
    ));

    for i in 1..NUM_PARTICLES {
        commands.spawn((
            BoidEntity(i as usize),
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(boid_mesh.clone()),
                material: boid_material.clone(),
                ..Default::default()
            },
        ));
    }
}

fn move_entities(
    time: Res<Time>,
    mut parameters: ResMut<Parameters>,
    mut worker: ResMut<AppComputeWorker<BoidWorker>>,
    mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>,
) {
    if !worker.ready() {
        return;
    }

    // for x in worker.read_vec::<Density>("density") {
    //     println!("{:?}", x);
    // }

    let boids = worker.read_vec::<Particle>("particles_dst");

    parameters.delta_time = time.delta_seconds() * 0.1;
    worker.write("params", parameters.as_ref());

    q_boid
        .par_iter_mut()
        .for_each(|(mut transform, boid_entity)| {
            transform.translation = boids[boid_entity.0].position.extend(0.0);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{app::AppExit, render::RenderPlugin, window::ExitCondition, winit::WinitPlugin};
    use rand::Rng;

    fn random_points(num_points: usize) -> Vec<Vec2> {
        let mut rng = rand::thread_rng();

        let mut points = Vec::new();
        for _ in 0..num_points {
            points.push(Vec2::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-3.0..3.0),
            ))
        }
        points
    }

    type CellKey = u32;
    type ParticleIndex = u32;

    #[derive(ShaderType, Pod, Zeroable, Clone, Copy, Debug, Default)]
    #[repr(C)]
    struct SpatialIndexEntry {
        cell_key: CellKey,
        particle_index: ParticleIndex,
    }

    struct SpatialIndexWorker;

    impl ComputeWorker for SpatialIndexWorker {
        fn build(world: &mut World) -> AppComputeWorker<Self> {
            let params = world.get_resource::<Parameters>().unwrap().clone();
            const NUM_PARTICLES: usize = 32;

            let mut entries = Vec::with_capacity(NUM_PARTICLES);
            entries.resize(
                NUM_PARTICLES,
                SpatialIndexEntry {
                    cell_key: 0,
                    particle_index: 21,
                },
            );

            // let mut rng = rand::thread_rng();
            // let unif = Uniform::new_inclusive(-1.0, 1.0);

            let mut positions = Vec::with_capacity(NUM_PARTICLES);
            for i in 0..NUM_PARTICLES {
                positions.push(Vec2::new(
                    i as f32,
                    (i / 2) as f32,
                    // (i % 3) as f32 * (params.particle_radius * 3.0) + 1.0 + params.x_min,
                    // (i / 3) as f32 * (params.particle_radius * 3.0) + 1.0 + params.y_min,
                ));
            }

            let mut start_indices: Vec<u32> = Vec::with_capacity(NUM_PARTICLES);
            start_indices.resize(NUM_PARTICLES, u32::MAX);

            AppComputeWorkerBuilder::new(world)
                .add_uniform("params", &params)
                .add_staging("positions", &positions)
                .add_staging("entries", &entries)
                .add_staging("start_indices", &start_indices)
                .add_pass::<shaders::SpatialComputeEntriesShader>(
                    [NUM_PARTICLES as u32 / 256 + 1, 1, 1],
                    &["params", "positions", "entries"],
                )
                .add_pass::<shaders::SpatialSortEntriesShader>(
                    [1, 1, 1],
                    &["entries"],
                )
                .add_pass::<shaders::SpatialComputeStartIndices>(
                    [NUM_PARTICLES as u32 / 256 + 1, 1, 1],
                    &["entries", "start_indices"],
                )
                .one_shot()
                .build()
        }
    }

    fn start_compute_worker(mut worker: ResMut<AppComputeWorker<SpatialIndexWorker>>) {
        // assert!(worker.ready());
        println!("Starting compute worker");
        worker.execute();
    }

    fn print_compute_shader_results(
        worker: ResMut<AppComputeWorker<SpatialIndexWorker>>,
        mut exit: EventWriter<AppExit>,
    ) {
        if !worker.ready() {
            println!("Compute worker not ready");
            return;
        }

        println!("Done.");
        println!("Entries");
        for (i, x) in worker.read_vec::<SpatialIndexEntry>("entries").into_iter().enumerate() {  
            println!("{}: {:?}", i, x);
        }
        println!("Start indices");
        for (i, x) in worker.read_vec::<u32>("start_indices").into_iter().enumerate() {
            println!("{}: {:?}", i, x);
        }
        exit.send(AppExit);

        // for x in worker.read_vec::<Density>("density") {
        //     println!("{:?}", x);
        // }

        // let boids = worker.read_vec::<Particle>("particles_dst");

        // parameters.delta_time = time.delta_seconds() * 0.1;
        // worker.write("params", parameters.as_ref());

        // q_boid
        //     .par_iter_mut()
        //     .for_each(|(mut transform, boid_entity)| {
        //         transform.translation = boids[boid_entity.0].position.extend(0.0);
        //     });
    }

    #[test]
    fn test_gpu_sort() {
        let mut app = App::new();
        app.add_plugins((DefaultPlugins
            .set(WindowPlugin {
                // primary_window: None,
                // exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(WinitPlugin {
                run_on_any_thread: true,
            }),))
            .add_plugins(AppComputePlugin)
            .add_plugins(AppComputeWorkerPlugin::<SpatialIndexWorker>::default())
            .insert_resource(Parameters::default())
            .add_systems(Startup, start_compute_worker)
            .add_systems(Update, print_compute_shader_results);

        shaders::ParticleShader::load_shader(&mut app);
        shaders::KernelShader::load_shader(&mut app);

        // Spatial index
        shaders::SpatialCommonShader::load_shader(&mut app);
        shaders::SpatialComputeEntriesShader::load_shader(&mut app);
        shaders::SpatialSortEntriesShader::load_shader(&mut app);
        shaders::SpatialComputeStartIndices::load_shader(&mut app);


        app.run();
    }
}
