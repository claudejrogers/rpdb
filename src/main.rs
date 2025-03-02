use clap::{Parser, Subcommand, ValueEnum};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
mod atom;
use atom::{AtomCollection, RecordType};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, ValueEnum)]
enum FileKind {
    Pdb,
    Cif,
}

/// CLI utilities for PDB files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Extract substructures from a PDB
    #[command(arg_required_else_help = true)]
    Parse {
        /// Path to PDB
        path: String,

        /// Record type to extract
        #[arg(short, long, default_value = "atom")]
        record_type: RecordType,

        /// Chain to select, [A-Z]
        #[arg(short, long)]
        chain: Option<Vec<char>>,

        /// Residue to select
        #[arg(short = 'R', long)]
        res: Option<String>,

        /// Move coordinates to center on origin
        #[arg(long, default_value_t = false)]
        center: bool,
    },

    /// Download structure files
    #[command(arg_required_else_help = true)]
    Fetch {
        /// Name(s) of structure file(s)
        name: Vec<String>,

        /// Kind of structure file
        #[arg(short, long, default_value = "pdb")]
        kind: FileKind,

        #[arg(long, default_value_t = false)]
        compress: bool,
    },
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

async fn download_structure(
    client: reqwest::Client,
    name: &str,
    ext: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = format!("{}.{}", name, ext);
    println!("Downloading {}", path);
    let url = format!("https://files.rcsb.org/download/{}", path);
    let resp = client.get(url).send().await?;
    if resp.status().is_success() {
        let mut file = tokio::fs::File::create(&path).await?;
        let body = resp.bytes().await?;
        file.write_all(&body).await?;
        println!("Saved {}", path)
    } else {
        println!("Download failed for {}: {:?}", path, resp.status());
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Parse {
            path,
            record_type,
            chain,
            res,
            center,
        } => {
            let record = match record_type {
                RecordType::Hetatm => "HETATM ",
                RecordType::Atom => "ATOM  ",
            };

            let res = res.clone();
            let chains = get_chain_mask(&chain);

            let mut collection = AtomCollection::new(record_type);
            let mut connections = HashMap::<u32, String>::new();
            if let Ok(lines) = read_lines(path) {
                for line in lines.map_while(Result::ok) {
                    if line.starts_with(record) {
                        collection
                            .add_atom(&line, |a| {
                                let test_chain = (chains & (1 << ((a.chain as u32) % 32))) != 0;
                                res.as_ref()
                                    .map_or(test_chain, |name| test_chain && (a.res_name == *name))
                            })
                            .unwrap();
                    } else if line.starts_with("CONECT") && (record_type == RecordType::Hetatm) {
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
            if center {
                collection.center_to_origin();
            }
            collection.output();
            if record_type == RecordType::Hetatm {
                for atom in &collection.entries {
                    if let Some(line) = connections.get(&atom.id) {
                        println!("{}", line);
                    }
                }
            }
        }

        Commands::Fetch {
            name,
            kind,
            compress,
        } => {
            let ext = match kind {
                FileKind::Pdb => {
                    if compress {
                        "pdb.gz"
                    } else {
                        "pdb"
                    }
                }
                FileKind::Cif => {
                    if compress {
                        "cif.gz"
                    } else {
                        "cif"
                    }
                }
            };
            let client = reqwest::Client::builder()
                .build()
                .expect("Failed to create HTTP client");

            let handles = name
                .into_iter()
                .map(|name| {
                    let clientc = client.clone();
                    tokio::spawn(async move { download_structure(clientc, &name, &ext).await })
                })
                .collect::<Vec<_>>();

            for handle in handles {
                let _ = handle.await.expect("Failed to complete download");
            }
        }
    }
}
