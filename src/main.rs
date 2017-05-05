extern crate specs;
extern crate time;

struct CompAABB (f32, f32, f32, f32);
impl specs::Component for CompAABB {
  type Storage = specs::VecStorage<CompAABB>;
}

struct CompVel (f32, f32);
impl specs::Component for CompVel {
  type Storage = specs::VecStorage<CompVel>;
}

#[derive(Clone)]
struct GlobalState { 
  /// Previous time in ns - unspecified epoch
  prev_time: u64, 
  /// Frame delta in ns
  delta: u64, 
}

impl GlobalState {
  /// Get the current frame delta in seconds.
  pub fn get_delta_in_s(&self) -> f32 {
    return self.delta as f32 / 1000000000.0f32;
  }
}

/// System for updating position according to velocity.
struct SysPosUpdate;
impl specs::System<GlobalState> for SysPosUpdate {
  fn run(&mut self, arg: specs::RunArg, state: GlobalState) {
    use specs::Join;
    let (mut all_aabb, all_vel, entities) = arg.fetch(|w| {
      (w.write::<CompAABB>(), w.read::<CompVel>(), w.entities())
    });
    for (e_id, aabb, vel) in (&entities, &mut all_aabb, &all_vel).join() {
      aabb.0 += vel.0 * (state.get_delta_in_s());
      aabb.1 += vel.1 * (state.get_delta_in_s());
    }
  }
}


fn main() {
  let mut global_state = GlobalState { delta: 0, prev_time: time::precise_time_ns() };
  let mut planner : specs::Planner<GlobalState> = {
    let mut w = specs::World::new();
    w.register::<CompAABB>();
    w.register::<CompVel>();
    w.create_now().with(CompAABB(0.0, 0.0, 32.0, 32.0)).with(CompVel(5.0, 5.0)).build();
    specs::Planner::new(w)
  };

  planner.add_system::<SysPosUpdate>(SysPosUpdate, "update", 0);

  loop {
    use specs::{Gate, Join};

    global_state.delta = time::precise_time_ns() - global_state.prev_time;
    global_state.prev_time = time::precise_time_ns();

    planner.dispatch(global_state.clone());
    planner.wait();

    let w = planner.mut_world();
    let pos_storage = w.read().pass();
    for e in w.entities().join() {
      let comp : &CompAABB = pos_storage.get(e.clone()).unwrap();
      println!("{}, {} - delta = {}", comp.0, comp.1, global_state.get_delta_in_s());
    }
  }
}
