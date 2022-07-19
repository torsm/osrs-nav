# osrs-nav/generator

Generates a NavGrid from game cache

## Running

```
USAGE:
    generator [OPTIONS] --cache <CACHE> --xteas <XTEAS> --output <OUTPUT>

OPTIONS:
    -c, --cache <CACHE>      Directory containing cache files
        --config <CONFIG>    YAML file with generator configuration
        --edges <EDGES>      YAML file with custom edges
    -h, --help               Print help information
    -o, --output <OUTPUT>    File that the generated NavGrid is serialized into
    -x, --xteas <XTEAS>      JSON file containing XTEA keys for the selected cache
```

The cache directory is the directory containing files like `main_file_cache.dat2` and `main_file_cache.idx_`.

The XTEAs file must match the cache's revision, get them from https://archive.runestats.com/osrs/xtea/ 