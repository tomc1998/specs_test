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

/// System for updating position according to velocity.
struct SysPosUpdate;
impl specs::System<GlobalState> for SysPosUpdate {
  fn run(&mut self, arg: specs::RunArg, state: GlobalState) {
    let (mut aabb, vel) = arg.fetch(|w| {
      (w.write_resource::<CompAABB>(), w.read_resource::<CompVel>())
    });
    aabb.0 += vel.0;
    aabb.1 += vel.1;
  }
}


fn main() {
  let mut global_state = GlobalState { delta: time::precise_time_ns(), prev_time: 0 };
  let mut planner : specs::Planner<GlobalState> = {
    let mut w = specs::World::new();
    w.register::<CompAABB>();
    w.register::<CompVel>();
    w.create_now().with(CompAABB(0.0, 0.0, 32.0, 32.0)).with(CompVel(5.0, 5.0)).build();
    specs::Planner::new(w)
  };

  loop {
    use specs::{Gate, Join};

    global_state.delta = time::precise_time_ns() - global_state.prev_time;
    global_state.prev_time = time::precise_time_ns();

    planner.add_system(SysPosUpdate, "update", 0);

    planner.dispatch(global_state.clone());

    let w = planner.mut_world();
    let pos_storage = w.read().pass();
    for e in w.entities().join() {
      let comp : &CompAABB = pos_storage.get(e.clone()).unwrap();
      println!("{}, {}", comp.0, comp.1);
    }
  }
}
