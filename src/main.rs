#[macro_use]
extern crate glium;
extern crate specs;
extern crate time;

pub mod component;
pub mod renderer;
pub mod state;

use component::*;
use state::GlobalState;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::Event;

/// System for updating position according to velocity.
struct SysPosUpdate;
impl specs::System<GlobalState> for SysPosUpdate {
  fn run(&mut self, arg: specs::RunArg, state: GlobalState) {
    use specs::Join;
    let (mut all_aabb, all_vel) = arg.fetch(|w| {
      (w.write::<CompAABB>(), w.read::<CompVel>())
    });
    for (aabb, vel) in (&mut all_aabb, &all_vel).join() {
      aabb.0 += vel.0 * (state.get_delta_in_s());
      aabb.1 += vel.1 * (state.get_delta_in_s());
    }
  }
}

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
    w.register::<CompVel>();
    w.register::<CompColor>();
    w.create_now().with(CompAABB(0.0, 0.0, 32.0, 32.0)).with(CompColor(0.0, 1.0, 0.0, 1.0)).build();
    specs::Planner::new(w)
  };

  let mut renderer = renderer::Renderer::new(&display);

  planner.add_system::<SysPosUpdate>(SysPosUpdate, "update", 0);
  planner.add_system::<renderer::SysRenderer>(renderer::SysRenderer::new(&renderer), "render", 0);

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
