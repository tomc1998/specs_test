//! A module for simulating rigid bodies in the game world.

use specs;
use component::*;
use state::GlobalState;

#[derive(Clone)]
pub struct RigidBody;

impl specs::System<GlobalState> for RigidBody {
  fn run(&mut self, arg: specs::RunArg, g: GlobalState) {
    let (mut all_aabb, mut all_body) = arg.fetch(|w| {
      (w.write::<CompAABB>(), w.write::<CompBody>())
    });

    use specs::Join;
    for (aabb, body) in (&mut all_aabb, &mut all_body).join() {
      let d = g.get_delta_in_s();
      let d2 = d.powi(2);
      println!("{}", body.flags);
      let gravity = if body.flags & BODY_GRAVITY > 0 {9.8} else {0.0};
      aabb.0[0] += body.vel[0]*d + d2*(body.acc[0])/2.0;
      aabb.0[1] += body.vel[1]*d + d2*(body.acc[1] + gravity)/2.0;
      body.vel[0] += (body.acc[0])*d;
      body.vel[1] += (body.acc[1] + gravity)*d;
    }
  }
}
