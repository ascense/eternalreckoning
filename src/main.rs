type Backend = rendy::vulkan::Backend;

use rendy::{
    factory::Factory,
    wsi::winit,
};

fn run(
    window: worldclient::window::Window,
    mut factory: Factory<Backend>,
    mut families: rendy::command::Families<Backend>,
    mut scene: worldclient::renderer::scene::Scene,
    graph: worldclient::renderer::RenderGraph<Backend>,
) -> Result<(), failure::Error> {
    let started = std::time::Instant::now();

    let mut frame = 0u64;
    let mut period = started;
    let mut graph = Some(graph);

    let mouse_sens = worldclient::input::MouseSensitivity::new(3.15);
    let mut mouse_euler = worldclient::input::MouseEuler::new();

    window.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                },
                _ => {},
            },
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    mouse_euler.update(delta, &mouse_sens);
                },
                _ => {},
            },
            winit::event::Event::EventsCleared => {
                let rotation = nalgebra::Rotation3::from_euler_angles(
                    mouse_euler.pitch as f32,
                    mouse_euler.yaw as f32,
                    0.0,
                );
                let position = nalgebra::Translation3::new(0.0f32, -1.0f32, 0.0f32);
                let translation = nalgebra::Translation3::new(0.0f32, 0.0f32, 10.0f32);
                scene.camera.set_view(
                    nalgebra::Projective3::identity() * position * rotation.inverse() * translation
                );

                factory.maintain(&mut families);

                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &scene);
                    frame += 1;
                }

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
            },
            _ => {},
        }

        if *control_flow == winit::event_loop::ControlFlow::Exit && graph.is_some() {
            graph.take().unwrap().dispose(&mut factory, &scene);
        }
    });

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

    let window = worldclient::window::Window::new();

    log::info!("Initializing rendering pipeline...");

    let aspect = window.get_aspect_ratio() as f32;

    let marker_reader = std::io::BufReader::new(
        std::fs::File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/marker.wc1"
        ))
        .unwrap()
    );

    let floor_reader = std::io::BufReader::new(
        std::fs::File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/floor.wc1"
        ))
        .unwrap()
    );

    let mut scene = worldclient::renderer::scene::Scene {
        camera: worldclient::renderer::scene::Camera::new(aspect),
        ui: worldclient::renderer::scene::UI::new(aspect),
        objects: vec![
            worldclient::renderer::scene::Object {
                mesh: worldclient::loaders::mesh_from_wc1(marker_reader)
                    .unwrap()
                    .build()
                    .unwrap(),
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(0.0, 0.0, 0.0),
            },
            worldclient::renderer::scene::Object {
                mesh: worldclient::loaders::mesh_from_wc1(floor_reader)
                    .unwrap()
                    .build()
                    .unwrap(),
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(0.0, 0.0, 0.0),
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

    run(window, factory, families, scene, graph).unwrap();
}
