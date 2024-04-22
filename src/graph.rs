use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
struct NodeHandle<T> {
    id: usize,
    node: Rc<RefCell<Node<T>>>,
}

impl<T> NodeHandle<T> {
    fn new(id: usize, node: Node<T>) -> Self {
        NodeHandle {
            id,
            node: Rc::new(RefCell::new(node)),
        }
    }
}

impl<T> Clone for NodeHandle<T> {
    fn clone(&self) -> Self {
        NodeHandle {
            id: self.id,
            node: Rc::clone(&self.node),
        }
    }
}

impl<T> Eq for NodeHandle<T> {}
impl<T> PartialEq for NodeHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node) && self.id == other.id
    }
}

#[derive(Debug)]
struct Node<T> {
    body: T,
    inbound: Vec<NodeHandle<T>>,
    outbound: Vec<NodeHandle<T>>,
}

impl<T> Node<T> {
    fn new(body: T) -> Self {
        Self {
            body,
            inbound: Vec::new(),
            outbound: Vec::new(),
        }
    }
}

struct Builder<T> {
    node_handles: Vec<NodeHandle<T>>,
}

impl<T> Builder<T>
where
    T: Debug,
{
    fn new() -> Self {
        Self {
            node_handles: Vec::new(),
        }
    }

    fn node(&mut self, body: T) -> NodeHandle<T> {
        let node = Node::new(body);
        let handle = NodeHandle::new(self.node_handles.len(), node);

        self.node_handles.push(NodeHandle::clone(&handle));
        handle
    }

    fn connect(&mut self, tail: &NodeHandle<T>, head: &NodeHandle<T>) {
        tail.node
            .borrow_mut()
            .outbound
            .push(NodeHandle::clone(head));

        head.node.borrow_mut().inbound.push(NodeHandle::clone(tail));
    }

    fn build(&self) -> Result<Graph<T>, String> {
        let sorted = topological_sort(&self.node_handles);

        Ok(Graph {
            layered_nodes: Vec::new(),
        })
    }
}

fn topological_sort<T: Debug>(node_handles: &Vec<NodeHandle<T>>) -> Result<Vec<usize>, String> {
    if node_handles.is_empty() {
        return Err("Empty nodes list".to_string());
    }

    // Now it's time to perform a Depth-First Search
    // https://en.wikipedia.org/wiki/Topological_sorting?useskin=vector#Depth-first_search

    let mut sorted = Vec::new();
    let mut marks = vec![(false, false); node_handles.len()];

    let start_indices: Vec<usize> = node_handles
        .iter()
        .enumerate()
        .filter(|(_, handle)| handle.node.borrow().inbound.is_empty())
        .map(|(i, _)| i)
        .collect();

    if start_indices.is_empty() {
        return Err("No start nodes detected, graph may be cyclical or empty".to_string());
    }

    for &i in start_indices.iter() {
        match visit(VisitProps {
            i,
            marks: &mut marks,
            handles: node_handles,
            sorted: &mut sorted,
        }) {
            Ok(_) => continue,
            Err(i) => {
                return Err(format!(
                    "Failure during DFS from index {i}: {:?}",
                    node_handles[i]
                ))
            }
        }
    }

    struct VisitProps<'a, T> {
        i: usize,
        marks: &'a mut Vec<(bool, bool)>,
        handles: &'a Vec<NodeHandle<T>>,
        sorted: &'a mut Vec<usize>,
    }

    fn visit<T>(
        VisitProps {
            i,
            marks,
            handles,
            sorted,
        }: VisitProps<T>,
    ) -> Result<(), usize> {
        let perm_mark = marks[i].0;
        let temp_mark = marks[i].1;

        if perm_mark {
            return Ok(());
        }
        if temp_mark {
            return Err(i);
        }

        marks[i].1 = true;

        let outbound_indices = handles[i]
            .node
            .borrow()
            .outbound
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>();

        for m_i in outbound_indices {
            match visit(VisitProps {
                i: m_i,
                marks,
                handles,
                sorted,
            }) {
                Ok(_) => continue,
                Err(i) => return Err(i),
            }
        }

        marks[i].1 = false;
        marks[i].0 = true;
        sorted.push(i);

        Ok(())
    }

    sorted.reverse();
    Ok(sorted)
}

struct Graph<T> {
    layered_nodes: Vec<Vec<T>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_connection() {
        let mut builder = Builder::new();
        let n1 = builder.node("node1");
        let n2 = builder.node("node2");
        builder.connect(&n1, &n2);

        assert_eq!(n1, n2.node.borrow().inbound[0]);
        assert_eq!(n2, n1.node.borrow().outbound[0]);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_topological_sort_success() {
//         let mut builder = Builder::<&str>::new();
//
//         // Add vertices
//         let v0 = builder.add_vertex("vertex0"); // start node
//         let v1 = builder.add_vertex("vertex1");
//         let v2 = builder.add_vertex("vertex2"); // start node
//
//         // Create edges
//         // v0 -> v1, v2 -> v1
//         builder.connect(v0, v1);
//         builder.connect(v2, v1);
//
//         // Build the graph
//         let graph = builder.build().unwrap();
//
//         // The sorted nodes should be either [v2, v0, v1] or [v0, v2, v1]
//         assert_eq!(graph.vertices.len(), 3);
//         assert!(graph.vertices[2] == "vertex1");
//         assert!(graph.vertices.contains(&"vertex0"));
//         assert!(graph.vertices.contains(&"vertex2"));
//     }
//
//     #[test]
//     fn test_topological_sort_with_single_node() {
//         let mut builder = Builder::<&str>::new();
//
//         // Add a single vertex
//         builder.add_vertex("vertex0");
//
//         // Build the graph
//         let graph = builder.build().unwrap();
//
//         // Check if the single vertex is correctly identified as a start vertex
//         assert_eq!(graph.vertices, vec!["vertex0"]);
//     }
//
//     #[test]
//     fn test_topological_sort_failure_due_to_cycle() {
//         let mut builder = Builder::<&str>::new();
//
//         // Add vertices
//         let v0 = builder.add_vertex("vertex0");
//         let v1 = builder.add_vertex("vertex1");
//
//         // Create a cycle: v0 -> v1 -> v0
//         builder.connect(v0, v1);
//         builder.connect(v1, v0);
//
//         // Attempt to build the graph, expecting failure
//         assert!(builder.build().is_err());
//     }
//
//     #[test]
//     fn test_complex_dag() {
//         let mut builder = Builder::<&str>::new();
//
//         // Add vertices
//         let v0 = builder.add_vertex("vertex0"); // start node
//         let v1 = builder.add_vertex("vertex1"); // start node
//         let v2 = builder.add_vertex("vertex2");
//         let v3 = builder.add_vertex("vertex3");
//         let v4 = builder.add_vertex("vertex4");
//         let v5 = builder.add_vertex("vertex5"); // start node
//
//         // Create complex edges
//         builder.connect(v0, v2);
//         builder.connect(v1, v2);
//         builder.connect(v1, v3);
//         builder.connect(v2, v4);
//         builder.connect(v3, v4);
//         builder.connect(v5, v3);
//
//         // Build the graph
//         let graph = builder.build().unwrap();
//
//         // The exact order can vary but must satisfy the dependencies
//         assert_eq!(graph.vertices.len(), 6);
//         let idx_v2 = graph.vertices.iter().position(|&x| x == "vertex2").unwrap();
//         let idx_v3 = graph.vertices.iter().position(|&x| x == "vertex3").unwrap();
//         let idx_v4 = graph.vertices.iter().position(|&x| x == "vertex4").unwrap();
//
//         // Ensure the topological order is correct
//         assert!(graph.vertices.iter().position(|&x| x == "vertex0").unwrap() < idx_v2);
//         assert!(graph.vertices.iter().position(|&x| x == "vertex1").unwrap() < idx_v2);
//         assert!(graph.vertices.iter().position(|&x| x == "vertex1").unwrap() < idx_v3);
//         assert!(idx_v2 < idx_v4);
//         assert!(idx_v3 < idx_v4);
//     }
//
//     #[test]
//     fn test_long_chain() {
//         let mut builder = Builder::<String>::new();
//
//         let mut vs = Vec::new();
//         // Add vertices in a linear chain
//         for i in 0..10 {
//             let handle = builder.add_vertex(format!("vertex{}", i));
//             vs.push(handle);
//         }
//
//         // Create a chain of edges v0 -> v1 -> v2 -> ... -> v9
//         for i in 0..vs.len() - 1 {
//             builder.connect(vs[i], vs[i + 1]);
//         }
//
//         // Build the graph
//         let graph = builder.build().unwrap();
//
//         // Check if the vertices are in exact order
//         let expected_order: Vec<String> = (0..10).map(|i| format!("vertex{}", i)).collect();
//         assert_eq!(graph.vertices, expected_order);
//     }
//
//     #[test]
//     fn test_edges_remap_correctly() {
//         let mut builder = Builder::<&str>::new();
//         let v0 = builder.add_vertex("vertex0");
//         let v1 = builder.add_vertex("vertex1");
//         let v2 = builder.add_vertex("vertex2");
//
//         builder.connect(v0, v1);
//         builder.connect(v1, v2);
//         builder.connect(v0, v2);
//
//         let graph = builder.build().unwrap();
//
//         // Check if vertices are sorted correctly
//         assert!(graph.vertices.contains(&"vertex0"));
//         assert!(graph.vertices.contains(&"vertex1"));
//         assert!(graph.vertices.contains(&"vertex2"));
//
//         // Check if edges are correctly remapped
//         let expected_edges = vec![(0, 1), (1, 2), (0, 2)];
//         assert_eq!(graph.edges, expected_edges);
//     }
// }
