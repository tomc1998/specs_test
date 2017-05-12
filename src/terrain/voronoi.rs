use renderer::{RendererController};
use std::collections::VecDeque;
use cgmath::Vector2;

/// Struct representing a vertex in the graph
#[derive(Clone, Debug)]
struct Node {
  /// A list of neighbours to this node. They are represented as an index into
  /// a list of nodes.
  neighbour_list: Vec<usize>,
  pos: [f32; 2],
}

/// An event in fortune's algorithm
enum Event {
  /// A site event.
  /// # Parameters
  /// * Position of event
  Site([f32; 2]),

  /// A circle event.
  /// # Parameters
  /// * Position of event
  /// * Radius of circle
  /// * ID of the arc
  Circle([f32; 2], f32, u32),
}

type EventQueue = VecDeque<Event>;

/// Arc struct, represents the parabola made up of the points equidistant from
/// a site and the scan line. 
#[derive(Clone, Debug)]
struct Arc {
  /// A unique ID for this arc, used to identify it
  id: u32,
  /// The coordinates of the point this is formed from
  p: [f32; 2],
  /// The vertex that the left side of this arc is drawing an edge from, if any
  l_vert : Option<usize>,
  /// The vertex that the right side of this arc is drawing an edge from, if any
  r_vert : Option<usize>,
}

pub fn voronoi(points: &Vec<[f32; 2]>, r: RendererController) {
  // Create event queue
  let mut ev_queue = EventQueue::new();

  // Draw sites
  for p in points { r.rect(&[p[0], p[1], 2.0, 2.0], &[1.0, 0.0, 0.0, 1.0]); }

  // Sort site events by height, add them in increasing y
  {
    let mut points_clone = points.clone();
    let mut points_added = 0;
    while points_added < points.len() {
      let mut highest_ix = None; // Index of the highest site
      for (ii, p) in points_clone.iter().enumerate() {
        if highest_ix.is_none() || p[1] > (points_clone[highest_ix.unwrap()] as [f32; 2])[1] { 
          highest_ix = Some(ii);
        }
      }
      if highest_ix.is_none() { break; } // No points left to add

      let highest_ix = highest_ix.unwrap();

      // Found highest, now add to event queue
      ev_queue.push_back(Event::Site(points_clone[highest_ix].clone()));

      // Remove this point from the cloned list of points
      points_clone.remove(highest_ix);
      points_added += 1;
    }
  }

  // Process events in a loop
  let mut curr_arc_id = 0;
  let mut arc_list = Vec::new(); // A list of arcs processed
  let mut node_list = Vec::new(); // A list of nodes (vertices)
  while ev_queue.len() != 0 {
    let e = ev_queue.pop_front();
    match e.unwrap() {
      // Process site event, add a new arc to the list
      Event::Site(p) => {
        // If no arcs, just push
        if arc_list.is_empty() { 
          arc_list.push(Arc{ p: p, l_vert: None, r_vert: None, id: curr_arc_id }); 
          curr_arc_id += 1;
          continue; 
        }

        // Otherwise, find the arc that lowest on the vertical line where 
        // x = p.x. Remember, sweep line is at p.y.
        // Let p = The point of the site, a = the point making the arc.
        // The equation of the parabola made from site A is:
        // y = (p.y^2 + a.y^2 + (a.x - x)^2)/(2(a.y - p.y))
        // The exact y value where the new site's parabola intersects this arc
        // is obtained by substituting p.x for x.
        let mut lowest_arc_ix = 0;
        let mut lowest_height = 0.0;
        for ii in 0..arc_list.len() {
          // TODO Possible optimisation by discarding arcs based on half the y distance from a to p.
          // Calculate intersection, then if it's the lowest, set lowest_arc.
          let a = arc_list[ii].p;
          let intersection_y = (p[1].powi(2) + a[1].powi(2) + (a[0] - p[0]).powi(2))/(2.0 * (a[1] - p[1]));
          if lowest_height < intersection_y {
            lowest_arc_ix = ii; 
            lowest_height = intersection_y;
          }
        }

        // Now we have the lowest arc, split it into 3 new arcs
        let arc_2 = arc_list[lowest_arc_ix].clone();
        let new_arc = Arc { p: p, l_vert: None, r_vert: None, id: curr_arc_id };
        curr_arc_id += 1;

        // Insert these new arcs back into the list
        arc_list.insert(lowest_arc_ix+1, new_arc);
        arc_list.insert(lowest_arc_ix+2, arc_2);

        // Now check for circle events with the newly inserted site arc

        /// A function to check whether a given point and its neighbours will
        /// create a circle event. If so, returns the circle event.
        /// # Params
        /// * `arc_list` - The list of arcs
        /// * `p_ix` - An index into the list pointing to the arc to be checked
        /// * `scanline` - The Y position of the scanline (not the beach line)
        fn check_circle_event(arc_list: &Vec<Arc>, p_ix: usize, scanline: f32) -> Option<Event>{
          if p_ix == 0 || p_ix == arc_list.len() - 1 { return None; }
          // We have neighbours, so compute the circumcircle center and radius
          // Let a, b and c be triangle points
          // Get perp bisectors of ab and bc, get their intersection. This is the center.
          // Do a lot of rearranging and you get 2 huge equations to give you x, then y:
          let (a, b, c) = (arc_list[p_ix-1].p, arc_list[p_ix].p, arc_list[p_ix+1].p);
          let center_x = 
            ((b[1].powi(2) + a[0].powi(2) - a[1].powi(2) - b[0].powi(2))/(2.0*(b[1]-a[1]))
             - (c[1].powi(2) + b[0].powi(2) - b[1].powi(2) - c[0].powi(2))/(2.0*(c[1]-b[1])))
            / ((c[0] - b[0])/(c[1] - b[1]) - (b[0] - a[0])/(b[1] - a[1]));
          // Y = mx + c
          let center_y = ((b[0] - a[0])/(b[1] - a[1]))*center_x + 
            (b[1].powi(2) + a[0].powi(2) - a[1].powi(2) - b[0].powi(2))/(2.0*(b[1] - a[1]));

          // Calculate r with pythagoras
          let r = ((center_x - a[0]).powi(2) + (center_y - a[1]).powi(2)).sqrt();

          // Check whether the bottom of the circle is below the scan line
          return 
            if center_y + r > scanline {
              Some(Event::Circle([center_x, center_y], r, arc_list[p_ix].id ))
            }
            else { None }
        }

        /// A function to insert a circle event into the event queue in the right place
        fn insert_circle(circle: Event, ev_queue: &mut EventQueue) {
          match circle {
            Event::Circle(cp, cr, _) => {
              for ii in 0..ev_queue.len() {
                match ev_queue[ii] {
                  Event::Circle(p, r, _) => {
                    if p[1] + r > cp[1] + cr {
                      ev_queue.insert(ii, circle);
                      return;
                    }
                  }
                  Event::Site(p) => {
                    if p[1] > cp[1] + cr {
                      ev_queue.insert(ii, circle);
                      return;
                    }
                  }
                }
              }
              ev_queue.push_back(circle);
            }
            _ => ()
          }
        }

        let circle = check_circle_event(&arc_list, lowest_arc_ix, p[1]);
        if circle.is_some() { 
          // Find the right index to insert the circle
          let circle = circle.unwrap();
          insert_circle(circle, &mut ev_queue);
        }
        let circle = check_circle_event(&arc_list, lowest_arc_ix+2, p[1]);
        if circle.is_some() { 
          // Find the right index to insert the circle
          let circle = circle.unwrap();
          insert_circle(circle, &mut ev_queue);
        }
      }

      // Process circle event
      Event::Circle(p, rad, id) => { 
        let mut neighbours = [None; 2];

        // Draw circles
        r.rect(&[p[0], p[1]+rad, 2.0, 2.0], &[0.0, 1.0, 0.0, 1.0]); 

        let new_node = Node { neighbour_list: Vec::new(), pos: p.clone() };
        node_list.push(new_node);
        let new_node_ix = node_list.len() - 1;

        // Find the arc that just had its size reduced to 0
        for ii in 0..arc_list.len() {
          if arc_list[ii].id == id {
            // Get neighbour vertices for new node
            if arc_list[ii].l_vert.is_some() { neighbours[0] = Some(arc_list[ii].l_vert.unwrap()); }
            if arc_list[ii].r_vert.is_some() { neighbours[1] = Some(arc_list[ii].r_vert.unwrap()); }
            // Add connections to the old vertices
            if neighbours[0].is_some() { node_list[neighbours[0].unwrap()].neighbour_list.push(new_node_ix); }
            if neighbours[1].is_some() { node_list[neighbours[1].unwrap()].neighbour_list.push(new_node_ix); }
            // Set the arcs to the left & the right's new v_right and v_left
            if ii > 0 { arc_list[ii-1].r_vert = Some(new_node_ix); }
            if ii < arc_list.len()-1 { arc_list[ii+1].l_vert = Some(new_node_ix); }
            // Remove the arc
            arc_list.remove(ii);
            break;
          }
        }

        // Add neighbours to the new node's neighbour list
        if neighbours[0].is_some() { node_list[new_node_ix].neighbour_list.push(neighbours[0].unwrap()); }
        if neighbours[1].is_some() { node_list[new_node_ix].neighbour_list.push(neighbours[1].unwrap()); }
      }
    }
  }

  // Draw graph
  // We don't really care about 'overdraw', so we can double up edge drawing. It's only debug
  for n in &node_list {
    for n2 in &n.neighbour_list {
      r.line(Vector2::new(n.pos[0], n.pos[1]), 
             Vector2::new(node_list[*n2].pos[0], node_list[*n2].pos[1]), 
             1.0, 
             [0.0, 1.0, 1.0, 1.0]);
    }
  }
}
