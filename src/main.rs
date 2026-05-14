use anyhow::Result;
use clap::Parser as ClapParser;

mod atomdb;
mod graph;
mod joinengine;
mod justification;
mod output;
mod program;
mod solver;

#[derive(ClapParser, Debug)]
#[command(name = "asp_solver", about = "Explainable ASP solver with justification graphs")]
struct Args {
    #[arg(short, long)]
    input: std::path::PathBuf,

    #[arg(short, long, default_value = "text")]
    format: String,

    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let program = program::parse(&std::fs::read_to_string(args.input)?)?;
    let db = atomdb::AtomDB::new(program.ground_atoms.clone());
    let engine = joinengine::ReteJoinEngine::new(&db, &program);
    let graph: graph::JustificationGraph = graph::build_justification_graph(&program, &engine);
    solver::solve(&program, db, graph);
    Ok(())
}
