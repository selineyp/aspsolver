use std::collections::HashMap;

use crate::program::{Rule, Atom, GroundAtom, Literal, PredicateArg};

pub trait JoinEngine {
    fn rules_watching(&self, atom: &GroundAtom) -> Vec<&Rule>;
    fn on_new_atom(&mut self, _atom: &GroundAtom) {}
}

pub struct Substitution {
    pub mapping: HashMap<String, String>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution { mapping: HashMap::new() }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.mapping.insert(key, value);
    }
}

pub fn apply(atom: &Atom, subst: &Substitution) -> GroundAtom {
    let args = atom.args.iter().map(|arg| match arg {
        PredicateArg::Variable(v) => Literal { name: subst.mapping.get(&v.name).cloned().unwrap_or(v.name.clone()) },
        PredicateArg::Literal(l) => l.clone(),
    }).collect();
    GroundAtom { name: atom.predicate.name.clone(), args, negated: atom.negated }
}

pub fn get_substitution(atom: &Atom, ground_atom: &GroundAtom) -> Option<Substitution> {
    if atom.args.len() != ground_atom.args.len() {
        return None;
    }
    let mut subst = Substitution::new();
    for (arg, g_arg) in atom.args.iter().zip(ground_atom.args.iter()) {
        match arg {
            PredicateArg::Variable(v) => {
                if let Some(existing) = subst.mapping.get(&v.name) {
                    if existing != &g_arg.name {
                        return None; // same variable bound to different values
                    }
                }
                subst.insert(v.name.clone(), g_arg.name.clone());
            }
            PredicateArg::Literal(l) => {
                if l.name != g_arg.name {
                    return None;
                }
            }
        }
    }
    Some(subst)
}

pub fn compute_matches(rule: &Rule, atom: &GroundAtom) -> Vec<Substitution> {
    rule.body.iter()
        .filter(|body_atom| body_atom.predicate.name == atom.name)
        .filter_map(|body_atom| get_substitution(body_atom, atom))
        .collect()
}

pub fn get_head_substitutions(rule: &Rule, ground_atom: &GroundAtom) -> Option<(Substitution, bool)> {
    for body_atom in &rule.body {
        if body_atom.predicate.name == ground_atom.name {
            if let Some(subst) = get_substitution(body_atom, ground_atom) {
                return Some((subst, body_atom.negated));
            }
        }
    }
    None
}
