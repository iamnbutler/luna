#[derive(Debug)]
struct Transform {
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
}

#[derive(Debug)]
enum Element {
    Rectangle { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Element {
    fn calculate_bounds(&self, transform: &Transform) -> Boundary {
        match self {
            Element::Rectangle { width, height } => Boundary {
                x: transform.x,
                y: transform.y,
                half_width: width * transform.scale_x * 0.5,
                half_height: height * transform.scale_y * 0.5,
            },
            Element::Circle { radius } => {
                let scaled_radius = radius * transform.scale_x.max(transform.scale_y);
                Boundary {
                    x: transform.x,
                    y: transform.y,
                    half_width: scaled_radius,
                    half_height: scaled_radius,
                }
            }
        }
    }
}

#[derive(Debug)]
struct SceneNode {
    id: usize,
    transform: Transform,
    element: Option<Element>,
    children: Vec<SceneNode>,
}

#[derive(Debug, Clone, Copy)]
struct Boundary {
    x: f32,
    y: f32,
    half_width: f32,
    half_height: f32,
}

#[derive(Debug)]
struct QuadTree {
    boundary: Boundary,
    capacity: usize,
    points: Vec<(f32, f32, usize)>,
    divided: bool,
    northeast: Option<Box<QuadTree>>,
    northwest: Option<Box<QuadTree>>,
    southeast: Option<Box<QuadTree>>,
    southwest: Option<Box<QuadTree>>,
}

impl QuadTree {
    fn insert_with_bounds(&mut self, bounds: &Boundary, id: usize) -> bool {
        if !self.intersects(bounds) {
            return false;
        }

        if self.points.len() < self.capacity {
            // Store the center point of the bounds
            self.points.push((bounds.x, bounds.y, id));
            return true;
        }

        if !self.divided {
            self.subdivide();
        }

        // Try to insert into child nodes
        if let Some(ref mut quad) = self.northeast {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.northwest {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southeast {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southwest {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        false
    }

    fn new(boundary: Boundary, capacity: usize) -> Self {
        Self {
            boundary,
            capacity,
            points: Vec::new(),
            divided: false,
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
        }
    }

    fn subdivide(&mut self) {
        let x = self.boundary.x;
        let y = self.boundary.y;
        let hw = self.boundary.half_width / 2.0;
        let hh = self.boundary.half_height / 2.0;
        self.northeast = Some(Box::new(QuadTree::new(
            Boundary {
                x: x + hw,
                y: y - hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.northwest = Some(Box::new(QuadTree::new(
            Boundary {
                x: x - hw,
                y: y - hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.southeast = Some(Box::new(QuadTree::new(
            Boundary {
                x: x + hw,
                y: y + hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.southwest = Some(Box::new(QuadTree::new(
            Boundary {
                x: x - hw,
                y: y + hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.divided = true;
    }

    fn insert(&mut self, x: f32, y: f32, id: usize) -> bool {
        if !self.contains(x, y) {
            return false;
        }
        self.insert_point(x, y, id)
    }

    fn insert_point(&mut self, x: f32, y: f32, id: usize) -> bool {
        if self.points.len() < self.capacity {
            self.points.push((x, y, id));
            return true;
        }
        if !self.divided {
            self.subdivide();
        }
        if let Some(ref mut quad) = self.northeast {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.northwest {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southeast {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southwest {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        false
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.boundary.x - self.boundary.half_width
            && x <= self.boundary.x + self.boundary.half_width
            && y >= self.boundary.y - self.boundary.half_height
            && y <= self.boundary.y + self.boundary.half_height
    }

    fn intersects(&self, other: &Boundary) -> bool {
        !(other.x - other.half_width > self.boundary.x + self.boundary.half_width
            || other.x + other.half_width < self.boundary.x - self.boundary.half_width
            || other.y - other.half_height > self.boundary.y + self.boundary.half_height
            || other.y + other.half_height < self.boundary.y - self.boundary.half_height)
    }

    fn query_range(&self, query_range: &Boundary) -> Vec<(f32, f32, usize)> {
        let mut found = Vec::new();

        if !self.intersects(query_range) {
            return found;
        }

        for &(x, y, id) in &self.points {
            if x >= query_range.x - query_range.half_width
                && x <= query_range.x + query_range.half_width
                && y >= query_range.y - query_range.half_height
                && y <= query_range.y + query_range.half_height
            {
                found.push((x, y, id));
            }
        }

        if self.divided {
            if let Some(ref quad) = self.northeast {
                found.extend(quad.query_range(query_range));
            }
            if let Some(ref quad) = self.northwest {
                found.extend(quad.query_range(query_range));
            }
            if let Some(ref quad) = self.southeast {
                found.extend(quad.query_range(query_range));
            }
            if let Some(ref quad) = self.southwest {
                found.extend(quad.query_range(query_range));
            }
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_bounds() {
        // Test Rectangle bounds
        let rect = Element::Rectangle {
            width: 100.0,
            height: 50.0,
        };
        let rect_transform = Transform {
            x: 10.0,
            y: 20.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let rect_bounds = rect.calculate_bounds(&rect_transform);
        assert_eq!(rect_bounds.x, 10.0);
        assert_eq!(rect_bounds.y, 20.0);
        assert_eq!(rect_bounds.half_width, 50.0);
        assert_eq!(rect_bounds.half_height, 25.0);

        // Test Circle bounds
        let circle = Element::Circle { radius: 30.0 };
        let circle_transform = Transform {
            x: -10.0,
            y: -20.0,
            scale_x: 2.0,
            scale_y: 2.0,
            rotation: 0.0,
        };
        let circle_bounds = circle.calculate_bounds(&circle_transform);
        assert_eq!(circle_bounds.x, -10.0);
        assert_eq!(circle_bounds.y, -20.0);
        assert_eq!(circle_bounds.half_width, 60.0);
        assert_eq!(circle_bounds.half_height, 60.0);
    }


    fn count_points(qt: &QuadTree) -> usize {
        let mut count = qt.points.len();
        if qt.divided {
            if let Some(ref ne) = qt.northeast {
                count += count_points(ne);
            }
            if let Some(ref nw) = qt.northwest {
                count += count_points(nw);
            }
            if let Some(ref se) = qt.southeast {
                count += count_points(se);
            }
            if let Some(ref sw) = qt.southwest {
                count += count_points(sw);
            }
        }
        count
    }

    fn check_node(qt: &QuadTree) {
        for (x, y, _id) in &qt.points {
            assert!(*x >= qt.boundary.x - qt.boundary.half_width);
            assert!(*x <= qt.boundary.x + qt.boundary.half_width);
            assert!(*y >= qt.boundary.y - qt.boundary.half_height);
            assert!(*y <= qt.boundary.y + qt.boundary.half_height);
        }
        if qt.divided {
            if let Some(ref ne) = qt.northeast {
                check_node(ne);
            }
            if let Some(ref nw) = qt.northwest {
                check_node(nw);
            }
            if let Some(ref se) = qt.southeast {
                check_node(se);
            }
            if let Some(ref sw) = qt.southwest {
                check_node(sw);
            }
        }
    }

    #[test]
    fn test_insert_within_boundary() {
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 10.0,
                half_height: 10.0,
            },
            4,
        );
        let inserted = qt.insert(1.0, 1.0, 1);
        assert!(inserted);
        assert_eq!(qt.points.len(), 1);
    }

    #[test]
    fn test_insert_outside_boundary() {
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 10.0,
                half_height: 10.0,
            },
            4,
        );
        let inserted = qt.insert(20.0, 20.0, 1);
        assert!(!inserted);
        assert_eq!(qt.points.len(), 0);
    }

    #[test]
    fn test_division() {
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 10.0,
                half_height: 10.0,
            },
            1,
        );
        assert!(!qt.divided);
        let inserted1 = qt.insert(1.0, 1.0, 1);
        assert!(inserted1);
        assert!(!qt.divided);
        let inserted2 = qt.insert(-1.0, -1.0, 2);
        assert!(inserted2);
        assert!(qt.divided);
        assert!(qt.northeast.is_some());
        assert!(qt.northwest.is_some());
        assert!(qt.southeast.is_some());
        assert!(qt.southwest.is_some());
    }

    #[test]
    fn test_complex_division() {
        // Create a QuadTree covering a 32x32 area (centered at 0,0 with half-dimensions 16)
        // with a capacity of 4.
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 16.0,
                half_height: 16.0,
            },
            4,
        );

        // A set of points scattered over the canvas.
        // They are chosen so that multiple subdivisions will occur.
        let points = vec![
            (10.0, 10.0, 1),   // Should land in the NE region.
            (12.0, 12.0, 2),   // Further NE.
            (-10.0, 10.0, 3),  // NW.
            (-12.0, 12.0, 4),  // Further NW.
            (-10.0, -10.0, 5), // SW.
            (-12.0, -12.0, 6), // Further SW.
            (10.0, -10.0, 7),  // SE.
            (12.0, -12.0, 8),  // Further SE.
            (0.0, 0.0, 9),     // Center.
            (5.0, 5.0, 10),    // NE but closer to center.
            (-5.0, 5.0, 11),   // NW but closer.
        ];

        for (x, y, id) in points.iter() {
            qt.insert(*x, *y, *id);
        }

        // Verify that all points are accounted for.
        assert_eq!(count_points(&qt), points.len());

        // Verify that each point is stored within a node that contains it.
        check_node(&qt);
    }

    #[test]
    fn test_insert_element_with_bounds() {
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 100.0,
                half_height: 100.0,
            },
            4,
        );

        // Create a rectangle element
        let rect = Element::Rectangle {
            width: 20.0,
            height: 10.0,
        };
        let rect_transform = Transform {
            x: 30.0,
            y: 40.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let rect_bounds = rect.calculate_bounds(&rect_transform);
        
        // Insert rectangle
        assert!(qt.insert_with_bounds(&rect_bounds, 1));

        // Create a circle element
        let circle = Element::Circle { radius: 15.0 };
        let circle_transform = Transform {
            x: -20.0,
            y: -30.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let circle_bounds = circle.calculate_bounds(&circle_transform);

        // Insert circle
        assert!(qt.insert_with_bounds(&circle_bounds, 2));

        // Verify elements were inserted by querying their regions
        let rect_query = qt.query_range(&rect_bounds);
        assert!(rect_query.iter().any(|&(_, _, id)| id == 1));

        let circle_query = qt.query_range(&circle_bounds);
        assert!(circle_query.iter().any(|&(_, _, id)| id == 2));
    }

    #[test]
    fn test_scene_node_with_quadtree() {
        // Create a QuadTree with sufficient size to hold our scene
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 100.0,
                half_height: 100.0,
            },
            4,
        );

        // Create scene nodes with different elements
        let rect_node = SceneNode {
            id: 1,
            transform: Transform {
                x: 25.0,
                y: 25.0,
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(Element::Rectangle {
                width: 30.0,
                height: 20.0,
            }),
            children: vec![],
        };

        let circle_node = SceneNode {
            id: 2,
            transform: Transform {
                x: -25.0,
                y: -25.0,
                scale_x: 2.0,
                scale_y: 2.0,
                rotation: 0.0,
            },
            element: Some(Element::Circle { radius: 10.0 }),
            children: vec![],
        };

        // Calculate bounds and insert elements into QuadTree
        if let Some(ref element) = rect_node.element {
            let bounds = element.calculate_bounds(&rect_node.transform);
            assert!(qt.insert_with_bounds(&bounds, rect_node.id));
        }

        if let Some(ref element) = circle_node.element {
            let bounds = element.calculate_bounds(&circle_node.transform);
            assert!(qt.insert_with_bounds(&bounds, circle_node.id));
        }

        // Query for elements and verify they're found
        let query_rect = Boundary {
            x: 25.0,
            y: 25.0,
            half_width: 20.0,
            half_height: 20.0,
        };
        let rect_results = qt.query_range(&query_rect);
        assert!(rect_results.iter().any(|&(_, _, id)| id == rect_node.id));

        let query_circle = Boundary {
            x: -25.0,
            y: -25.0,
            half_width: 25.0,
            half_height: 25.0,
        };
        let circle_results = qt.query_range(&query_circle);
        assert!(circle_results.iter().any(|&(_, _, id)| id == circle_node.id));
    }

    #[test]
    fn test_query_range() {
        let mut qt = QuadTree::new(
            Boundary {
                x: 0.0,
                y: 0.0,
                half_width: 16.0,
                half_height: 16.0,
            },
            4,
        );
        let points = vec![
            (5.0, 5.0, 1),
            (-5.0, -5.0, 2),
            (10.0, 10.0, 3),
            (12.0, 12.0, 4),
            (0.0, 0.0, 5),
        ];
        for (x, y, id) in points {
            qt.insert(x, y, id);
        }
        let query_range = Boundary {
            x: 5.0,
            y: 5.0,
            half_width: 3.0,
            half_height: 3.0,
        };
        let found_points = qt.query_range(&query_range);
        assert_eq!(found_points.len(), 1);
        assert_eq!(found_points[0].2, 1);
    }
}
