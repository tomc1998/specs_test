struct CompVel (f32, f32);
impl specs::Component for CompVel {
  type Storage = specs::VecStorage<CompVel>;
}


