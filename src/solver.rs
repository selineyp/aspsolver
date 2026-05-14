use crate::program::{Program, GroundAtom, RuleKind};
use crate::atomdb::AtomDB;
use crate::graph::JustificationGraph;
use crate::joinengine::{JoinEngine, apply, compute_matches};
use crate::joinengine::ReteJoinEngine;

pub enum PropagationResult {
    Fixpoint,
    ChoicePoint,
    Conflict,
}

pub struct UNSATCore {
    pub core: Vec<GroundAtom>,
}

pub struct Success<'a> {
    pub answer_set: Vec<GroundAtom>,
    pub justification_graph: JustificationGraph<'a>,
}

pub enum SolveResult<'a> {
    UNSATCore(UNSATCore),
    Success(Success<'a>),
}

pub fn propagate(
    mut worklist: Vec<GroundAtom>,
    engine: &mut impl JoinEngine,
    db: &mut AtomDB,
    _graph: &mut JustificationGraph<'_>,
) -> PropagationResult {
    while let Some(atom) = worklist.pop() {
        engine.on_new_atom(&atom);
        db.insert(&atom);
        let rules: Vec<crate::program::Rule> = engine.rules_watching(&atom)
            .iter()
            .map(|r| (*r).clone())
            .collect();
        for rule in &rules {
            for subst in compute_matches(rule, &atom) {
                match rule.kind {
                    RuleKind::Fact => unreachable!(),
                    RuleKind::Regular => {
                        if let Some(either::Either::Left(head_atom)) = &rule.head {
                            let derived = apply(head_atom, &subst);
                            worklist.push(derived);
                        }
                    }
                    RuleKind::Choice => return PropagationResult::ChoicePoint,
                    RuleKind::Constraint => return PropagationResult::Conflict,
                }
            }
        }
    }
    PropagationResult::Fixpoint
}

pub fn solve<'a>(program: &'a Program, mut db: AtomDB, mut graph: JustificationGraph<'a>) -> SolveResult<'a> {
    let mut engine = ReteJoinEngine::new(&db, program);
    let worklist: Vec<GroundAtom> = program.ground_atoms.clone();
    loop {
        let result = propagate(worklist.clone(), &mut engine, &mut db, &mut graph);
        match result {
            PropagationResult::Conflict => {
                return SolveResult::UNSATCore(UNSATCore { core: db.atoms() });
            }
            PropagationResult::ChoicePoint => {
                // TODO: implement backtracking search over choice points
                return SolveResult::UNSATCore(UNSATCore { core: vec![] });
            }
            PropagationResult::Fixpoint => {
                return SolveResult::Success(Success {
                    answer_set: db.atoms(),
                    justification_graph: graph,
                });
            }
        }
    }
}
