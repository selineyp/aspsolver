
#[derive(Clone, Debug)]
pub struct Predicate {
    pub name: String,
    pub arity: u8
}

#[derive(Clone, Debug)]
pub struct ChoiceStatement {
    pub min: u16,
    pub max: u16,
    pub choice_atom: Atom,
    pub body_atoms: Vec<Atom>,
}

use regex::Regex;
use either::Either;
use std::sync::atomic::{AtomicUsize, Ordering};


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Literal{
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable{
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub predicate: Predicate,
    pub args: Vec<PredicateArg>,
    pub negated: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PredicateArg{
    Literal(Literal),
    Variable(Variable),
}

#[derive(Clone, Debug)]
pub enum RuleKind {
    Fact,
    Regular,
    Choice,
    Constraint,
}

static RULE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct Rule {
    pub head: Option<Either<Atom, ChoiceStatement>>,
    pub body: Vec<Atom>,
    pub kind: RuleKind,
    pub id: usize,
}

impl Rule {
    pub fn new(head: Option<Either<Atom, ChoiceStatement>>, body: Vec<Atom>, kind: RuleKind) -> Self {
        let id = RULE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Rule { head, body, kind, id }
    }
}

pub enum ParsedRule {
    Rule(Rule),
    GroundAtom(GroundAtom),
}

pub struct Program {
    pub rules: Vec<Rule>,
    pub ground_atoms: Vec<GroundAtom>,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct GroundAtom {
    pub name: String,
    pub args: Vec<Literal>,
    pub negated: bool,
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    parts.push(s[start..].trim());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}
/// Parses ASP program from a string source.
/// Returns a `Program` struct representing the parsed rules.
pub fn parse(source: &str) -> anyhow::Result<Program> {
    let mut rules = Vec::new();
    let mut ground_atoms = Vec::new();

    for (_lineno, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }
        let rules_in_line = line.split('.').filter(|s| !s.trim().is_empty());
        for rule in rules_in_line {
            let rule: &str = rule.trim();
            let parsed = parse_rule(rule);
            match parsed {
                ParsedRule::GroundAtom(atom) => ground_atoms.push(atom),
                ParsedRule::Rule(rule) => {rules.push(rule);}
            }
        }
    }
    Ok(Program { rules:rules, ground_atoms:ground_atoms })
}

fn parse_atom(s: &str) -> Atom {
    let negated = s.starts_with("not ");
    let parsed_name = s.trim().trim_start_matches("not ").strip_suffix(")").unwrap().split('(').nth(0).unwrap_or("").trim();
    let parsed_arguments: Vec<PredicateArg> = s
        .trim()
        .strip_suffix(")")
        .unwrap()
        .split('(')
        .nth(1)
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| match s.chars().next().unwrap_or(' ').is_uppercase() {
            true => PredicateArg::Variable(Variable { name: s.to_string() }),
            false => PredicateArg::Literal(Literal { name: s.to_string() }),
        })
        .collect();
    Atom { predicate: Predicate { name: parsed_name.to_string(), arity: parsed_arguments.len() as u8 }, args: parsed_arguments, negated }
}

fn parse_rule(s: &str) -> ParsedRule {
    let head = s.split(":-").next().unwrap().trim();
    let body_part = s.trim_end_matches('.').split(":-").nth(1).map(|s| s.trim());
    
    if !s.contains(":-") {
        // Handle ground atoms (facts)
        let negated = head.starts_with("not ");
        let clean_head = if negated { &head[4..] } else { head };
        let predicate_name = clean_head.split('(').next().unwrap_or("").trim();
        let args_str = clean_head.strip_prefix(&format!("{}(", predicate_name)).unwrap_or("");
        let args_str = args_str.strip_suffix(')').unwrap_or(args_str);
        let args: Vec<Literal> = if args_str.is_empty() {
            Vec::new()
        } else {
            args_str.split(',').map(|s| Literal { name: s.trim().to_string() }).collect()
        };
        return ParsedRule::GroundAtom(GroundAtom {
            name: predicate_name.to_string(),
            args,
            negated
        });
    }
    else {
        let parsed_atoms: Vec<&str> = split_top_level_commas(body_part.unwrap());
        let mut atoms: Vec<Atom> = Vec::new();
        for parsed_atom in parsed_atoms {
            let s = parsed_atom;
            atoms.push(parse_atom(s));
        }

        if head.is_empty() {
            // constraint rule
            return ParsedRule::Rule(Rule::new(None, atoms, RuleKind::Constraint));
        }
        let re = Regex::new(r"(?<min>\d+)\{(?<choice>.*)\}(?<max>\d+)").unwrap();
        let mut it=  re.captures_iter(head);
        let it_next = it.next();
        let is_head_choice = it_next.is_some() && it_next.as_ref().unwrap().name("choice").is_some();
        if is_head_choice {
            // choice rule head
            let grp: regex::Captures<'_> = it_next.unwrap();
            let choice_rule_head = grp["choice"].to_string();
            let choice_atom = choice_rule_head.split(':').nth(0).unwrap_or("").trim();
            let choice_body = choice_rule_head.split(':').nth(1).unwrap_or("").trim();
            let choice_atom = parse_atom(choice_atom);
            //assume only one predicate in choice body for now, TODO: handle multiple predicates in choice body
            let choice_body = parse_atom(choice_body);
            let choice_statement = ChoiceStatement {
                min: grp["min"].parse::<u16>().unwrap_or(0),
                max: grp["max"].parse::<u16>().unwrap_or(0),
                choice_atom: choice_atom,
                body_atoms: vec![choice_body],
            };
            return ParsedRule::Rule(Rule::new(Some(Either::Right(choice_statement)), atoms, RuleKind::Choice));
        }
    else{
        //parse head as predicate
        let head_predicate = parse_atom(head);
        return ParsedRule::Rule(Rule::new(Some(Either::Left(head_predicate)), atoms, RuleKind::Regular));
    }
  }
 }


#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_graph() {
        use super::*;
        let program = parse(
            "color(blue). color(green). color(red).
            node(1). node(2). node(3).
            edge(1,2). edge(2,3). edge(1,3).
            edge(2,1). edge(3,2). edge(3,1).
            1{colored(N,C):color(C)}1 :- node(N).
            :- colored(N,C), colored(M,C), edge(M,N).").unwrap();
        assert_eq!(program.ground_atoms.len(), 12);
        assert_eq!(program.rules.len(), 2);
        assert_eq!(program.rules[0].head.is_none(), false);
        assert_eq!(program.rules[0].body.len(), 1);
        assert_eq!(program.rules[1].id, program.rules[0].id + 1);
        match program.rules[0].head.as_ref().unwrap() {
            Either::Left(_) => {},
            Either::Right(choice) => {
                // Access choice fields here
                assert_eq!(choice.min, 1);
                assert_eq!(choice.max, 1);
            }
        }
        assert_eq!(program.rules[1].head.is_none(), true);
        assert_eq!(program.rules[1].body.len(), 3);
        assert_eq!(program.rules[1].body[0].predicate.name, "colored");
        assert_eq!(program.rules[1].body[1].args[0], PredicateArg::Variable(Variable { name: "M".to_string() }));
        assert_eq!(program.rules[1].body[1].args[1], PredicateArg::Variable(Variable { name: "C".to_string() }));
        assert_eq!(program.rules[1].body[1].args[0], PredicateArg::Variable(Variable { name: "M".to_string() }));
        assert_eq!(program.rules[1].body[1].args[1], PredicateArg::Variable(Variable { name: "C".to_string() }));
    }
}