# rpdb: A simple CLI tool for macromolecule structure files

```sh
    $ rpdb --help
    CLI utilities for PDB files

    Usage: rpdb <COMMAND>

    Commands:
      parse  Extract substructures from a PDB
      fetch  Download structure files
      help   Print this message or the help of the given subcommand(s)

    Options:
      -h, --help     Print help
      -V, --version  Print version
```

## Subcommands 

## Parse

``` sh
    $ rpdb parse --help
    
    Extract substructures from a PDB

    Usage: rpdb parse [OPTIONS] <PATH>

    Arguments:
      <PATH>  Path to PDB
    
    Options:
      -r, --record-type <RECORD_TYPE>  Record type to extract [default: atom] [possible values: atom, hetatm]
      -c, --chain <CHAIN>              Chain to select, [A-Z]
      -R, --res <RES>                  Residue to select
          --center                     Move coordinates to center on origin
      -h, --help                       Print help

```

A quick and dirty tool to extract ATOM/HETATM components from PDB files
and optionally translate the coordinates such that the compound is centered on
the origin.

## Fetch 

``` sh
    $ rpdb fetch --help

    Download structure files
    
    Usage: rpdb fetch [OPTIONS] [NAME]...
    
    Arguments:
      [NAME]...  Name(s) of structure file(s)
    
    Options:
      -k, --kind <KIND>  Kind of structure file [default: pdb] [possible values: pdb, cif]
          --compress
      -h, --help         Print help
  
```

Downloads files for given entry ids

## Examples

Extract the ATOM/TER entries for a structure

```sh
    # Print the ATOM/TER entries
    $ rpdb parse 4ith.pdb
    ATOM      1  N   ASN A   8     -38.324  10.915  18.564  1.00 92.45           N
    ATOM      2  CA  ASN A   8     -38.836  11.600  17.392  1.00 96.62           C
    ATOM      3  C   ASN A   8     -37.827  11.737  16.268  1.00 97.17           C
    ATOM      4  O   ASN A   8     -37.801  10.927  15.365  1.00 99.65           O
    ATOM      5  CB  ASN A   8     -39.389  12.961  17.755  1.00 99.84           C
    ATOM      6  CG  ASN A   8     -40.171  13.571  16.623  1.00102.64           C
    # Additional output... 
```


Extract HETATM and corresponding CONECT entries with chain == 'A' 
and residue name == "RCM", center the coordinates around the origin.
Save output to RCM_A.pdb

``` sh
    $ rpdb parse 4ith.pdb -r hetatm -c A --res RCM --center > RCM_A.pdb
```


Downloads PDB files for entry ids 7au1, 6jk9, 8h07

``` sh
    $ rpdb fetch 7au1 6jk9 8h07
```

Same as above, except `.cif.gz` files are downloaded

``` sh
    $ rpdb fetch 7au1 6jk9 8h07 -k cif --compress
```
