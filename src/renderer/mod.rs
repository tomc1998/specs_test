mod shader;

use specs;
use component::*;
use state::GlobalState;
use std::sync::mpsc;

use glium::{self, VertexBuffer};
use glium::backend::glutin_backend::GlutinFacade;

/// The constant size of the renderer's VBO in vertices (i.e. can contain 1024 vertices)
pub const VBO_SIZE : usize = 1024;

#[derive(Copy, Clone, Debug)]
struct Vertex {
  pos: [f32; 2],
  col: [f32; 4],
}
implement_vertex!(Vertex, pos, col);

pub struct Renderer {
  /// The VBO to use. This will have data buffered to it when render() is called.
  vbo: VertexBuffer<Vertex>,

  /// The program to use for rendering
  program: glium::Program,

  /// The vertex data to be draw when render() is called. Data is moved into
  /// this buffer when `recv_data()` is called, then moved to the VBO for
  /// rendering in `render()`.
  v_data: [Vertex; VBO_SIZE],

  /// A tuple containing a sender and receiver - used for sending data to
  /// the renderer from different threads to be stored in v_data for the
  /// render() function.
  v_channel_pair: (mpsc::Sender<Vec<Vertex>>, mpsc::Receiver<Vec<Vertex>>),

  /// The projection matrix used to render the game. 
  proj_mat: [[f32; 4]; 4],
}

impl Renderer {
  /// Create a new renderer.
  /// # Params
  /// * `display` - The glutin display (OpenGL Context)
  /// * `system` - The SysRenderer being used by the ECS. When rendering,
  ///              vertex data will be buffered from here.
  pub fn new(display: &GlutinFacade) -> Renderer {
    let (w, h) = display.get_window().unwrap().get_inner_size().unwrap();
    Renderer {
      vbo: VertexBuffer::empty_dynamic(display, VBO_SIZE).unwrap(),
      program: shader::get_program(display),
      v_data: [ Vertex { pos: [0.0; 2], col: [0.0; 4] }; VBO_SIZE ],
      v_channel_pair: mpsc::channel(),
      proj_mat: [[2.0/w as f32, 0.0,           0.0, -0.0],
                 [0.0,         -2.0/h as f32,  0.0,  0.0],
                 [0.0,          0.0,          -1.0,  0.0],
                 [-1.0,         1.0,           0.0,  1.0]],
    }
  }

  /// Buffer the vertex data received from the ECS render system
  /// (`SysRenderer`) to the VBO to be rendered. This should be called before
  /// `render()`.
  pub fn recv_data(&mut self) {
    let mut v_len : usize = 0; // The size of the data being buffered. If this exceeds
    // VBO_SIZE, no more data must be buffered.
    loop {
      let res = self.v_channel_pair.1.try_recv();
      if res.is_err() {
        // If the result of try_recv is an error, either all the sender's are
        // disconnected (not expected, as we own a sender) OR the channel is
        // empty, which means we've buffered all the data we can.
        match res.err().unwrap() {
          mpsc::TryRecvError::Empty => break,
          mpsc::TryRecvError::Disconnected => panic!("Vertex data senders disconnected!")
        }
      }
      // Copy data from the packet into v_data
      let data_packet = res.unwrap();

      for v in data_packet {
        self.v_data[v_len] = v;
        v_len += 1;

        // Check data packet won't be too long
        #[cfg(feature = "vbo_overflow_panic")]
        { if v_len >= self.v_data.len() { panic!("VBO Overflow"); } }
      }
    }
  }

  pub fn render<T : glium::Surface>(&mut self, target: &mut T) {

    // Empty indices - basically only rendering sprites, so no need to have it indexed.
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // Write the vertex data to the VBO
    self.vbo.write(&self.v_data);

    // Load the projection matrix into the uniforms
    let uniforms = uniform! {
      proj_mat: self.proj_mat,
    };

    // Draw everything!
    target.draw(&self.vbo, 
                &indices, 
                &self.program, 
                &uniforms, 
                &Default::default()).unwrap();
  }

  /// # Returns
  /// A Sender<Vertex> for sending vertex data to the renderer. When
  /// render() is called, this data will be rendered then cleared.
  fn get_v_sender(&self) -> mpsc::Sender<Vec<Vertex>> {
    return self.v_channel_pair.0.clone();
  }
}

/// The ECS system, which controls the buffering of vertex data into the
/// Renderer via a system of channels.
#[derive(Clone)]
pub struct SysRenderer {
  /// Vertex data sender - used to send to the renderer for rendering from
  /// a different thread.
  v_data_sender: mpsc::Sender<Vec<Vertex>>,
}

impl SysRenderer {
  /// Create a new renderer system, which will buffer data to the given
  /// renderer when ran.
  pub fn new(r: &Renderer) -> SysRenderer {
    SysRenderer { v_data_sender: r.get_v_sender() }
  }
}

impl<'a> specs::System<GlobalState> for SysRenderer {
  fn run(&mut self, arg: specs::RunArg, _: GlobalState) {
    let (all_col, all_aabb) = arg.fetch(|w|  {
      (w.read::<CompColor>(), w.read::<CompAABB>())
    });

    let mut data = vec![
      Vertex { pos: [0.0, 0.0], col: [1.0, 0.0, 0.0, 1.0] },
      Vertex { pos: [1.0, 0.0], col: [1.0, 1.0, 0.0, 1.0] },
      Vertex { pos: [1.0, 1.0], col: [1.0, 1.0, 1.0, 1.0] }];

    use specs::Join;
    for (col, aabb) in (&all_col, &all_aabb).join() {
      data.push( Vertex { 
        pos: [aabb.0, aabb.1], 
        col: [col.0, col.1, col.2, col.3] 
      });
      data.push( Vertex { 
        pos: [aabb.0 + aabb.2, aabb.1], 
        col: [col.0, col.1, col.2, col.3] 
      });
      data.push( Vertex { 
        pos: [aabb.0 + aabb.2, aabb.1 + aabb.3], 
        col: [col.0, col.1, col.2, col.3] 
      });

      data.push( Vertex { 
        pos: [aabb.0, aabb.1], 
        col: [col.0, col.1, col.2, col.3] 
      });
      data.push( Vertex { 
        pos: [aabb.0, aabb.1 + aabb.3], 
        col: [col.0, col.1, col.2, col.3] 
      });
      data.push( Vertex { 
        pos: [aabb.0 + aabb.2, aabb.1 + aabb.3], 
        col: [col.0, col.1, col.2, col.3] 
      });
    }


    // Send data through the mpsc channel
    self.v_data_sender.send(data).unwrap();
  }
}
