use std::borrow::Cow;
use std::collections::HashMap;
use crate::program::{GroundAtom, Program};
use crate::joinengine::{JoinEngine, get_substitution};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub rule: usize,
    pub support: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node<'a> {
    pub ground_atom: Cow<'a, GroundAtom>,
}

pub struct JustificationGraph<'a> {
    pub nodes: Vec<Node<'a>>,
    pub edges: Vec<Edge>,
}

pub fn build_justification_graph<'a>(program: &'a Program, engine: &impl JoinEngine) -> JustificationGraph<'a> {
    let mut graph = JustificationGraph { nodes: vec![], edges: vec![] };
    let mut node_dict: HashMap<GroundAtom, usize> = HashMap::new();

    graph.nodes.push(Node { ground_atom: Cow::Owned(GroundAtom { name: "false".into(), args: vec![], negated: false }) });

    for ground_atom in &program.ground_atoms {
        node_dict.insert(ground_atom.clone(), graph.nodes.len());
        graph.nodes.push(Node { ground_atom: Cow::Borrowed(ground_atom) });
    }
    for rule in &program.rules {
        match &rule.head {
            Some(either::Either::Left(head_atom)) => {
                for (h_atom, &h_idx) in &node_dict {
                    if h_atom.name != head_atom.predicate.name { continue; }
                    if get_substitution(head_atom, h_atom).is_none() { continue; }
                    for body_atom in &rule.body {
                        for (g_atom, &g_idx) in &node_dict {
                            if g_atom.name == body_atom.predicate.name {
                                graph.edges.push(Edge {
                                    from: g_idx,
                                    to: h_idx,
                                    rule: rule.id,
                                    support: !body_atom.negated,
                                });
                            }
                        }
                    }
                }
            }
            Some(either::Either::Right(_)) => {}
            None => {}
        }
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{atomdb::AtomDB, joinengine::ReteJoinEngine};
    #[test]
    fn test_justification_graph_basic() {
        let program = crate::program::parse(
            "color(1). colored(1,1). node(1). node(2).
            colored(X,X) :- color(X), not node(X).").unwrap();
        let db: AtomDB = AtomDB::new(program.ground_atoms.clone());
        let engine = ReteJoinEngine::new(&db, &program);
        let graph: JustificationGraph = build_justification_graph(&program, &engine);
        assert_eq!(graph.nodes.len(), 5); // false + color(1) + colored(1,1) + node(1) + node(2)
        assert_eq!(graph.edges.len(), 3); // color(1) -(+)-> colored(1,1), node(1) -(-)-> colored(1,1), node(2) -(+)-> colored(1,1)
        for edge in &graph.edges {
            match edge.from {
                1 => {assert_eq!(edge.to, 2);assert_eq!(edge.support, true)} // fact color(1)
                3 => {assert_eq!(edge.to, 2);assert_eq!(edge.support, false)} // fact node(1)
                4 => {assert_eq!(edge.to, 2);assert_eq!(edge.support, false)} // fact node(2)
                _ => panic!("Unexpected edge"),
            }
        }
    }
}
