<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

## Usage

```bash
find,filter and delete duplicate files

Usage: deduplicator [OPTIONS] [scan_dir_path]

Arguments:
  [scan_dir_path]  Run Deduplicator on dir different from pwd (e.g., ~/Pictures )

Options:
  -t, --types <TYPES>          Filetypes to deduplicate [default = all]
  -i, --interactive            Delete files interactively
  -m, --min-size <MIN_SIZE>    Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T) [default: 1b]
  -D, --max-depth <MAX_DEPTH>  Max Depth to scan while looking for duplicates
  -d, --min-depth <MIN_DEPTH>  Min Depth to scan while looking for duplicates
  -f, --follow-links           Follow links while scanning directories
  -s, --strict                 Guarantees that two files are duplicate (performs a full hash)
  -p, --progress               Show Progress spinners & metrics
  -h, --help                   Print help
  -V, --version                Print version
```
### Examples

```bash
# Scan for duplicates recursively from the current dir, only look for png, jpg & pdf file types & interactively delete files
deduplicator -t pdf,jpg,png -i

# Scan for duplicates recursively from the ~/Pictures dir, only look for png, jpeg, jpg & pdf file types & interactively delete files
deduplicator ~/Pictures/ -t png,jpeg,jpg,pdf -i

# Scan for duplicates in the ~/Pictures without recursing into subdirectories
deduplicator ~/Pictures --max-depth 0

# look for duplicates in the ~/.config directory while also recursing into symbolic link paths
deduplicator ~/.config --follow-links

# scan for duplicates that are greater than 100mb in the ~/Media directory
deduplicator ~/Media --min-size 100mb
```

## Installation
Currently, you can only install deduplicator using cargo package manager.

### Cargo
> [!WARNING] GxHash relies on aes hardware acceleration, so please set `RUSTFLAGS` to `"-C target-feature=+aes"` or `"-C target-cpu=native"` before
> installing.

#### install from crates.io
```bash
$ RUSTFLAGS="-C target-cpu=native" cargo install deduplicator
```

#### install from git
```bash
$ RUSTFLAGS="-C target-cpu=native" cargo install deduplicator --git https://github.com/sreedevk/deduplicator
```

## Performance
Deduplicator uses size comparison and [GxHash](https://docs.rs/gxhash/latest/gxhash/) to quickly check a large number of files to find duplicates. its also heavily parallelized. The default behavior of deduplicator is to only hash the first page (4K) of the file. This is to ensure that performance is the default priority. You can modify this behavior by using the `--strict` flag which will hash the whole file and ensure that 2 files are indeed duplicates. I'll add benchmarks in future versions.

## proposed
- [ ] parallelization
    - [ ] (scanning + processing sw + processing hw) & formatting & printing
    - [ ] scanning + processing sw + processing hw + formatting + printing
- [ ] max file path size should use the last set of duplicates
- [ ] add more unit tests
- [ ] restore json output (was removed in 0.3)
- [ ] fix memory leak on very large filesystems
    - [ ] maybe use a bloom filter
    - [ ] reduce FileInfo size
- [ ] output in a tree format
- [ ] tui
- [ ] add benchmarks
- [ ] change the default hashing method to include the first & last page of a file (8K)

## v0.3
- [x] parallelization
    - [x] (scanning) + (processing sw & processing hw & formatting & printing)
- [x] reduce cloning values on the heap
- [x] add a partial hashing mode (--strict)
- [x] add unit tests
- [x] add silent mode
- [x] update documentation
- [x] remove color output
- [x] progress bar improvements
    - [x] use progress bar groups
