use anyhow::{anyhow, Result};
use clap::ValueEnum;

#[derive(Debug, Copy, Clone, ValueEnum, PartialEq)]
pub enum RecordType {
    Atom,
    Hetatm,
}

pub struct Atom {
    pub record_type: RecordType,
    pub id: u32,
    pub name: String,
    pub alt_loc: char,
    pub res_name: String,
    pub chain: char,
    pub res_id: u32,
    pub icode: char,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub occupancy: f32,
    pub temp_factor: f32,
    pub element: String,
    pub charge: String,
}

impl Atom {
    pub fn from_line(line: &str) -> Result<Self> {
        let record_type = match line.get(..6).unwrap() {
            "ATOM  " => RecordType::Atom,
            "HETATM" => RecordType::Hetatm,
            _ => return Err(anyhow!("Not an atom entry")),
        };
        let id = line
            .get(6..11)
            .ok_or(anyhow!("Could not get id"))?
            .trim()
            .parse::<u32>()?;
        let name = String::from(
            line.get(12..16)
                .ok_or(anyhow!("Could not get name"))?
                .trim(),
        );
        let alt_loc = line
            .get(16..17)
            .ok_or(anyhow!("Could not get alt_loc"))?
            .chars()
            .next()
            .expect("Expected alt_loc is a char");
        let res_name = line.get(17..20).ok_or(anyhow!("Could not get res_name"))?;
        let res_name = String::from(res_name.trim());
        let chain = line
            .get(21..22)
            .ok_or(anyhow!("Could not get chain"))?
            .chars()
            .next()
            .expect("Expected chain is a char");
        let res_id = line
            .get(22..26)
            .ok_or(anyhow!("Could not get res_id"))?
            .trim()
            .parse::<u32>()?;
        let icode = line
            .get(26..27)
            .ok_or(anyhow!("Could not get icode"))?
            .chars()
            .next()
            .expect("Expected icode is a char");
        let x = line
            .get(30..38)
            .ok_or(anyhow!("Could not get x"))?
            .trim()
            .parse::<f32>()?;
        let y = line
            .get(38..46)
            .ok_or(anyhow!("Could not get y"))?
            .trim()
            .parse::<f32>()?;
        let z = line
            .get(46..54)
            .ok_or(anyhow!("Could not get z"))?
            .trim()
            .parse::<f32>()?;
        let occupancy = line
            .get(54..60)
            .ok_or(anyhow!("Could not get occupancy"))?
            .trim()
            .parse::<f32>()?;
        let temp_factor = line
            .get(60..66)
            .ok_or(anyhow!("Could not get occupancy"))?
            .trim()
            .parse::<f32>()?;
        let element = String::from(
            line.get(76..78)
                .ok_or(anyhow!("Could not get element"))?
                .trim(),
        );
        let charge = String::from(
            line.get(78..80)
                .ok_or(anyhow!("Could not get charge"))?
                .trim(),
        );

        Ok(Self {
            record_type,
            id,
            name,
            alt_loc,
            res_name,
            chain,
            res_id,
            icode,
            x,
            y,
            z,
            occupancy,
            temp_factor,
            element,
            charge,
        })
    }

    // ATOM   4047  OE2 GLU B 294     -31.789 -48.532  31.944  1.00 55.08           O
    // 012345678901234567890123456789012345678901234567890123456789012345678901234567890
    //           1         2         3         4         5         6         7         8
    pub fn to_string(&self) -> String {
        let space1 = if self.element.len() == 1 { " " } else { "" };
        let space2 = if self.element.len() == 1 { "" } else { " " };
        let record = match self.record_type {
            RecordType::Atom => "ATOM",
            RecordType::Hetatm => "HETATM",
        };
        format!(
            "{:6} {:>4} {}{:<3}{}{}{:>3} {}{:>4}{}   {:8.3}{:8.3}{:8.3}{:6.2}{:6.2}          {:>2}{:>2}",
            record,
            self.id,
            space1,
            self.name,
            space2,
            self.alt_loc,
            self.res_name,
            self.chain,
            self.res_id,
            self.icode,
            self.x,
            self.y,
            self.z,
            self.occupancy,
            self.temp_factor,
            self.element,
            self.charge,
        )
    }

    pub fn translate(&mut self, vec: &[f32; 3]) {
        self.x = self.x - vec[0];
        self.y = self.y - vec[1];
        self.z = self.z - vec[2];
    }
}

pub struct AtomCollection {
    pub record_type: RecordType,
    pub entries: Vec<Atom>,
}

impl AtomCollection {
    pub fn new(record: RecordType) -> Self {
        Self {
            record_type: record,
            entries: Vec::<Atom>::new(),
        }
    }

    pub fn add_atom<F>(&mut self, line: &str, f: F) -> Result<()>
    where
        F: Fn(&Atom) -> bool,
    {
        let atom = Atom::from_line(line)?;
        if f(&atom) {
            self.entries.push(atom);
        }
        Ok(())
    }

    fn ter_line(last_line: &Atom) -> String {
        let record = "TER";
        let id = last_line.id + 1;
        let space = "";
        format!(
            "{:6} {:>4}      {:>3} {}{:>4}{:54}",
            record, id, last_line.res_name, last_line.chain, last_line.res_id, space,
        )
    }

    pub fn output(&self) {
        for chunk in self.entries.windows(2) {
            match chunk {
                [prev, next] => {
                    println!("{}", prev.to_string());
                    if (prev.chain != next.chain) && self.record_type == RecordType::Atom {
                        println!("{}", Self::ter_line(prev));
                    }
                }
                _ => {}
            }
        }
        if let Some(last) = self.entries.last() {
            println!("{}", last.to_string());
            if self.record_type == RecordType::Atom {
                println!("{}", Self::ter_line(last));
            }
        }
    }

    pub fn center_of_mass(&self) -> [f32; 3] {
        let n = self.entries.len() as f32;
        self.entries
            .iter()
            .map(|e| [e.x, e.y, e.z])
            .reduce(|acc, e| [acc[0] + e[0], acc[1] + e[1], acc[2] + e[2]])
            .map(|e| [e[0] / n, e[1] / n, e[2] / n])
            .expect("Should have coordinates")
    }

    pub fn center_to_origin(&mut self) {
        let com = self.center_of_mass();
        self.entries.iter_mut().for_each(|e| e.translate(&com));
    }
}
