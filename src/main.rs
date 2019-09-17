type Backend = rendy::vulkan::Backend;

use rendy::{
    factory::Factory,
    wsi::winit,
};

fn run(
    window: &mut worldclient::window::Window,
    factory: &mut Factory<Backend>,
    families: &mut rendy::command::Families<Backend>,
    scene: &mut worldclient::renderer::scene::Scene,
    mut graph: worldclient::renderer::RenderGraph<Backend>,
) -> Result<(), failure::Error> {
    let started = std::time::Instant::now();

    let mut frames = 0u64..;
    let mut period = started;

    let mut closed = false;

    let mouse_sens = worldclient::input::MouseSensitivity::new(3.15);
    let mut mouse_euler = worldclient::input::MouseEuler::new();

    for frame in &mut frames {
        factory.maintain(families);
        window.poll_events(|event| {
            match event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    window_id: _,
                } => dbg!(closed = true),
                winit::Event::DeviceEvent {
                    event: winit::DeviceEvent::MouseMotion { delta },
                    device_id: _,
                } => {
                    mouse_euler.update(delta, &mouse_sens);
                },
                _ => (),
            }
        });

        let rotation = nalgebra::Rotation3::from_euler_angles(
            mouse_euler.pitch as f32,
            mouse_euler.yaw as f32,
            0.0,
        );
        let translation = nalgebra::Translation3::new(0.0f32, 0.0f32, 10.0f32);
        scene.camera.set_view(
            nalgebra::Projective3::identity() * rotation.inverse() * translation
        );

        graph.run(factory, families, &scene);

        if period.elapsed() >= std::time::Duration::new(5, 0) {
            period = std::time::Instant::now();
            let elapsed = started.elapsed();
            let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;
            
            log::info!(
                "Elapsed: {:?}. Frames: {}. FPS: {}",
                elapsed,
                frame,
                frame * 1_000_000_000 / elapsed_ns
            );
        }

        if closed {
            break;
        }
    }

    graph.dispose(factory, &scene);
    Ok(())
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("worldclient", log::LevelFilter::Trace)
        .init();

    let config: rendy::factory::Config = Default::default();
    let (mut factory, mut families): (Factory<Backend>, _) =
        rendy::factory::init(config).unwrap();

    log::info!("Creating window...");

    let mut window = worldclient::window::Window::new();

    log::info!("Initializing rendering pipeline...");

    let aspect = window.get_aspect_ratio() as f32;

    let mut scene = worldclient::renderer::scene::Scene {
        camera: worldclient::renderer::scene::Camera::new(aspect),
        ui: worldclient::renderer::scene::UI::new(aspect),
        objects: vec![
            worldclient::renderer::scene::Object {
                mesh: worldclient::renderer::mesh::MeshBuilder::new()
                    .with_vertices(&[
                        [0.0, -0.5, 0.0],
                        [0.5, 0.5, 0.0],
                        [-0.5, 0.5, 0.0],
                    ])
                    .with_colors(&[
                        [1.0, 0.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                    ])
                    .build()
                    .unwrap(),
                position: nalgebra::Point3::new(0.0, 0.0, 0.0),
            },
            worldclient::renderer::scene::Object {
                mesh: worldclient::renderer::mesh::MeshBuilder::new()
                    .with_vertices(&[
                        [0.0, 1.0, 0.0],
                        [-10.0, 1.0, 0.0],
                        [0.0, 1.0, 10.0],
                        [0.0, 1.0, 0.0],
                        [0.0, 1.0, 10.0],
                        [10.0, 1.0, 0.0],
                        [0.0, 1.0, 0.0],
                        [10.0, 1.0, 0.0],
                        [0.0, 1.0, -10.0],
                        [0.0, 1.0, 0.0],
                        [0.0, 1.0, -10.0],
                        [-10.0, 1.0, 0.0],
                    ])
                    .with_colors(&[
                        [0.5, 0.5, 0.5, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.5, 0.5, 0.5, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.5, 0.5, 0.5, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.5, 0.5, 0.5, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                        [0.25, 0.25, 0.25, 1.0],
                    ])
                    .build()
                    .unwrap(),
                position: nalgebra::Point3::new(0.0, 0.0, 0.0),
            },
        ],
    };

    let graph = worldclient::renderer::RenderGraph::new(
        &mut factory,
        &mut families,
        &mut scene,
        &window,
    );

    log::info!("Entering main loop");

    run(&mut window, &mut factory, &mut families, &mut scene, graph).unwrap();
}
