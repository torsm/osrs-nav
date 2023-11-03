# osrs-nav/generator

Generates a NavGrid from game cache

## Running

```
Usage: generator.exe [OPTIONS] --input <INPUT> --output <OUTPUT>                                                    
                                                                                                                    
Options:                                                                                                            
  -i, --input <INPUT>    Directory containing cache files and xteas
  -o, --output <OUTPUT>  File that the generated NavGrid is serialized into
      --edges <EDGES>    YAML file with custom edges
      --config <CONFIG>  YAML file with generator configuration
  -h, --help             Print help
```

The cache directory is the directory containing files like `main_file_cache.dat2` and `main_file_cache.idx_`.

The XTEAs file must match the cache's revision, get them from https://archive.runestats.com/osrs/xtea/ 