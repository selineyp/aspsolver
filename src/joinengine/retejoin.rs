use std::collections::HashMap;
use crate::program::{GroundAtom, Rule};
use crate::joinengine::JoinEngine;
use crate::atomdb::AtomDB;
use either::Either;

pub struct ReteJoinEngine {
    rules_watching: HashMap<String, Vec<Rule>>,
}

impl ReteJoinEngine {
    pub fn new(atomdb: &AtomDB, program: &crate::program::Program) -> Self {
        let mut engine = ReteJoinEngine { rules_watching: HashMap::new() };
        for atom in atomdb.atoms() {
            for rule in program.rules.iter() {
                for body_atom in &rule.body {
                    if body_atom.predicate.name == atom.name {
                        engine.rules_watching.entry(atom.name.clone()).or_default().push(rule.clone());
                    }
                }
            }
        }
        engine
    }
}

impl JoinEngine for ReteJoinEngine {
    fn rules_watching(&self, atom: &GroundAtom) -> Vec<&Rule> {
        self.rules_watching.get(&atom.name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}