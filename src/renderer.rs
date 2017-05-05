use state::GlobalState;
use specs;
use component::*;

pub struct SysRenderer;
impl specs::System<GlobalState> for SysRenderer {
  fn run(&mut self, arg: specs::RunArg, state: GlobalState) {
    let (all_col, all_aabb) = arg.fetch(|w|  {
      (w.read::<CompColor>(), w.read::<CompAABB>())
    });
  }
}
