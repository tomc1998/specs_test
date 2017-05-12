#[macro_use]
extern crate glium;
extern crate specs;
extern crate time;
extern crate rand;
extern crate cgmath;

pub mod component;
pub mod renderer;
pub mod state;
pub mod terrain;
pub mod physics;

use component::*;
use state::GlobalState;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::Event;
use rand::{Rng, StdRng};

fn init_display() -> GlutinFacade {
    use glium::DisplayBuild;
    use glium::glutin::{Api, GlRequest, GlProfile};
    glium::glutin::WindowBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 0)))
      .with_gl_profile(GlProfile::Core)
      .build_glium().unwrap()
}

fn main() {
  let display = init_display();

  let mut global_state = GlobalState { delta: 0, prev_time: time::precise_time_ns() };

  // Create ECS
  let mut planner : specs::Planner<GlobalState> = {
    let mut w = specs::World::new();
    w.register::<CompAABB>();
    w.register::<CompBody>();
    w.register::<CompColor>();
    w.create_now().with(CompAABB([0.0, 0.0, 32.0, 32.0]))
      .with(CompColor([0.0, 1.0, 0.0, 1.0]))
      .with(CompBody{vel: [0.0, 0.0], acc: [0.5, 0.3], mass: 5.0, flags: BODY_GRAVITY})
      .build();
    specs::Planner::new(w)
  };

  let mut renderer = renderer::Renderer::new(&display);

  planner.add_system::<renderer::SysRenderer>(renderer::SysRenderer::new(&renderer), "render", 0);
  planner.add_system::<physics::RigidBody>(physics::RigidBody, "ph_rigid_body", 0);

  let mut rng = StdRng::new().unwrap();
  let mut voronoi_sites = vec![];
  for ii in 0..4 {
    for jj in 0..4 {
      voronoi_sites.push([100.0 + (ii as f32) * 50.0, 100.0 + (jj as f32) * 50.0 + ii as f32]);
    }
  }

  loop {
    // Check input
    for ev in display.poll_events() {
      match ev {
        Event::Closed => return,
          _ => ()
      }
    }

    // Calculate frame delta, store in global state object
    global_state.delta = time::precise_time_ns() - global_state.prev_time;
    global_state.prev_time = time::precise_time_ns();

    // Dispatch ECS with the global state object
    planner.dispatch(global_state.clone());
    planner.wait();

    // Add voronoi points test data
    terrain::voronoi::voronoi(&voronoi_sites, renderer.get_renderer_controller());

    // Receive any vertex data sent by the ECS
    renderer.recv_data();

    // Render everything
    use glium::Surface;
    let mut frame = display.draw();
    frame.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.render(&mut frame);
    frame.finish().unwrap();
  }
}
