use specs;

/// Color component. R, G, B, A format.
pub struct CompColor(f32, f32, f32, f32);
impl specs::Component for CompColor {
  type Storage = specs::VecStorage<CompColor>;
}
