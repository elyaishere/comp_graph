use std::{cell::RefCell, rc::Rc};


#[derive(Clone)]
pub struct Graph(Rc<RefCell<Node>>);

enum OperationType {
    Input(String),
    Add(Graph, Graph),
    Mul(Graph, Graph),
    Sin(Graph),
    PowF32(Graph, Graph),
}

pub struct Node {
    /// Operation performed by the node
    op_type: OperationType,
    /// Nodes which must be recalculated when the value in current node is changed
    dependent_nodes: Vec<Graph>,
    /// Cached value
    cache: Option<f32>,
}

impl Node {
    fn new(op_type: OperationType, cache: Option<f32>) -> Node {
        Node {
            op_type,
            dependent_nodes: vec![],
            cache,
        }
    }

    fn wrap(self) -> Graph {
        Graph(Rc::new(RefCell::new(self)))
    }
}

impl Graph {
    pub fn create_input<I: Into<String>>(val: I) -> Graph {
        Node::new(OperationType::Input(val.into()), None).wrap()
    }

    pub fn add(op1: Graph, op2: Graph) -> Graph {
        let node = Node::new(OperationType::Add(op1.clone(), op2.clone()), None).wrap();
        Self::add_dependent_node(&node, op1);
        Self::add_dependent_node(&node, op2);
        node
    }

    pub fn mul(op1: Graph, op2: Graph) -> Graph {
        let node = Node::new(OperationType::Mul(op1.clone(), op2.clone()), None).wrap();
        Self::add_dependent_node(&node, op1);
        Self::add_dependent_node(&node, op2);
        node
    }

    pub fn pow_f32(b: Graph, exp: Graph) -> Graph {
        let node = Node::new(OperationType::PowF32(b.clone(), exp.clone()), None).wrap();
        Self::add_dependent_node(&node, b);
        Self::add_dependent_node(&node, exp);
        node
    }

    pub fn sin(op: Graph) -> Graph {
        let node = Node::new(OperationType::Sin(op.clone()), None).wrap();
        Self::add_dependent_node(&node, op);
        node
    }

    fn traverse(node: &Graph) -> f32 {
        let mut node = node.0.as_ref().borrow_mut();
        if let &Some(cache) = &node.cache {
            return cache;
        }

        match &node.op_type {
            OperationType::Input(ref _s) => node.cache.unwrap(),
            OperationType::Add(op1, op2) => {
                let res = Self::traverse(op1) + Self::traverse(op2);
                node.cache.replace(res);
                res
            }
            OperationType::Mul(op1, op2) => {
                let res = Self::traverse(op1) * Self::traverse(op2);
                node.cache.replace(res);
                res
            }
            OperationType::Sin(op) => {
                let res = Self::traverse(op).sin();
                node.cache.replace(res);
                res
            }
            OperationType::PowF32(b, exp) => {
                let res = Self::traverse(b).powf(Self::traverse(exp));
                node.cache.replace(res);
                res
            }
        }
    }

    pub fn compute(&self) -> f32 {
        Self::traverse(self)
    }

    fn clear_cash(node: &Graph) {
        let mut node = node.0.as_ref().borrow_mut();
        let _ = node.cache.take();
        for dep in &node.dependent_nodes {
            Self::clear_cash(dep);
        }
    }

    pub fn set<I: Into<f32>>(&self, new_val: I) {
        let node = self.0.as_ref().borrow_mut();
        if let OperationType::Input(ref _s) = node.op_type {
            drop(node);
            Self::clear_cash(&self);
            self.0.as_ref().borrow_mut().cache.replace(new_val.into());
        }
    }

    fn add_dependent_node(&self, op: Graph) {
        let mut op = op.0.as_ref().borrow_mut();
        op.dependent_nodes.push(self.clone());
    }

}

#[cfg(test)]
mod test {
    use crate::Graph;


    /// Round to decimal digits
    fn round(x: f32, precision: u32) -> f32 {
        let m = 10i32.pow(precision) as f32;
        (x * m).round() / m
    }

    #[test]
    fn test() {
        let x1 = Graph::create_input("x1");
        let x2 = Graph::create_input("x2");
        let x3 = Graph::create_input("x3");
        let x4 = Graph::create_input("x4");

        let graph = Graph::add(
            x1.clone(),
            Graph::mul(
                x2.clone(),
                Graph::sin(
                    Graph::add(
                        x2.clone(),
                        Graph::pow_f32(x3.clone(), x4.clone())
                    )
                )
            )
        );

        x1.set(1f32);
        x2.set(2f32);
        x3.set(3f32);
        x4.set(3f32);

        let mut result = graph.compute();
        result = round(result, 5);
        assert_eq!(result, -0.32727);

        x1.set(2f32);
        x2.set(3f32);
        x3.set(4f32);
        result = graph.compute();
        result = round(result, 5);
        assert_eq!(result, -0.56656);
    }

}
