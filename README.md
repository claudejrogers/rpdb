# rpdb: A simple CLI tool to extract components from PDB files

This is a quick and dirty tool to extract ATOM/HETATM components from PDB files
and optionally translate the coordinates such that the compound is centered on
the origin.

``` sh
    $ rpdb --help
    Extract substructures from a PDB

    Usage: rpdb [OPTIONS] --path <PATH>

    Options:
    -p, --path <PATH>                Path to PDB
    -r, --record-type <RECORD_TYPE>  Record type to extract [default: atom] [possible values: atom, hetatm]
    -c, --chain <CHAIN>              Chain to select, [A-Z]
    -R, --res <RES>                  Residue to select
        --center                     Move coordinates to center on origin
    -h, --help                       Print help
    -V, --version                    Print version

```

## Examples

``` sh
    # Print the ATOM/TER
    $ rpdb -p 4ith.pdb

    # Extract HETATM and corresponding CONECT entries with chain == 'A' 
    # and residue name == "RCM", center the coordinates around the origin.
    # Save output to RCM_A.pdb
    $ rpdb -p 4ith.pdb -r hetatm -c A --res RCM --center > RCM_A.pdb
```

