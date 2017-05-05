use specs;

/// AABB component - X, Y, W, H format.
struct CompAABB (f32, f32, f32, f32);
impl specs::Component for CompAABB {
  type Storage = specs::VecStorage<CompAABB>;
}

struct CompVel (f32, f32);
impl specs::Component for CompVel {
  type Storage = specs::VecStorage<CompVel>;
}



