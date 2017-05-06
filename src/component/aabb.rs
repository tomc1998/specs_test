use specs;

/// AABB component - X, Y, W, H format.
pub struct CompAABB (pub f32, pub f32, pub f32, pub f32);
impl specs::Component for CompAABB {
  type Storage = specs::VecStorage<CompAABB>;
}

pub struct CompVel (pub f32, pub f32);
impl specs::Component for CompVel {
  type Storage = specs::VecStorage<CompVel>;
}



