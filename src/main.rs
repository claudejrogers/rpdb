use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
mod atom;
use atom::{AtomCollection, RecordType};
use std::collections::HashMap;

/// Extract substructures from a PDB
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to PDB
    #[arg(short, long)]
    path: String,

    /// Record type to extract
    #[arg(short, long, default_value = "atom")]
    record_type: Option<RecordType>,

    /// Chain to select, [A-Z]
    #[arg(short, long)]
    chain: Option<Vec<char>>,

    /// Residue to select
    #[arg(short = 'R', long)]
    res: Option<String>,

    /// Move coordinates to center on origin
    #[arg(long, default_value_t = false)]
    center: bool,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_chain_mask(chain: &Option<Vec<char>>) -> u32 {
    chain.as_ref().map_or(134217726, |list| {
        list.iter()
            .fold(0, |mask, c| mask | 1 << ((*c as u32) % 32))
    })
}

fn main() {
    let args = Args::parse();

    let record = match args.record_type {
        Some(RecordType::Hetatm) => "HETATM ",
        _ => "ATOM  ",
    };

    let res = args.res.clone();
    let chains = get_chain_mask(&args.chain);

    let mut collection =
        AtomCollection::new(args.record_type.expect("Record type should have a default"));
    let mut connections = HashMap::<u32, String>::new();
    if let Ok(lines) = read_lines(args.path) {
        for line in lines.map_while(Result::ok) {
            if line.starts_with(record) {
                collection
                    .add_atom(&line, |a| {
                        let test_chain = (chains & (1 << ((a.chain as u32) % 32))) != 0;
                        res.as_ref()
                            .map_or(test_chain, |name| test_chain && (a.res_name == *name))
                    })
                    .unwrap();
            } else if line.starts_with("CONECT") && (args.record_type == Some(RecordType::Hetatm)) {
                if let Ok(id) = line
                    .get(6..11)
                    .unwrap_or("Could not get conect ID")
                    .trim()
                    .parse::<u32>()
                {
                    connections.insert(id, line.clone());
                }
            }
        }
    }
    if args.center {
        collection.center_to_origin();
    }
    collection.output();
    if args.record_type == Some(RecordType::Hetatm) {
        for atom in &collection.entries {
            if let Some(line) = connections.get(&atom.id) {
                println!("{}", line);
            }
        }
    }
}
