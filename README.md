<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

## Usage

```bash
find,filter,delete duplicate files

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

### Cargo Install
#### Stable
> [!WARNING] Note from GxHash: GxHash relies on aes hardware acceleration, you must make sure the aes feature is enabled when building (otherwise it won't build). This can be done by setting the RUSTFLAGS environment variable to -C target-feature=+aes or -C target-cpu=native (the latter should work if your CPU is properly recognized by rustc, which is the case most of the time).
> please install version `0.2.1`  if you are unable to install `0.3.0`

```bash
$ RUSTFLAGS="-C target-cpu=native" cargo install deduplicator
```

#### Nightly

if you'd like to install with nightly features, you can use

```bash
$ cargo install --git https://github.com/sreedevk/deduplicator
```
Please note that if you use a version manager to install rust (like asdf), you need to reshim (`asdf reshim rust`).

### Linux (Pre-built Binary)

you can download the pre-built binary from the [Releases](https://github.com/sreedevk/deduplicator/releases) page.
download the `deduplicator-x86_64-unknown-linux-gnu.tar.gz` for linux. Once you have the tarball file with the executable,
you can follow these steps to install:

```bash
$ tar -zxvf deduplicator-x86_64-unknown-linux-gnu.tar.gz
$ sudo mv deduplicator /usr/bin/
```

### Mac OS (Pre-built Binary)

you can download the pre-build binary from the [Releases](https://github.com/sreedevk/deduplicator/releases) page.
download the `deduplicator-x86_64-apple-darwin.tar.gz` tarball for mac os. Once you have the tarball file with the executable, you can follow these steps to install:

```bash
$ tar -zxvf deduplicator-x86_64-unknown-linux-gnu.tar.gz
$ sudo mv deduplicator /usr/bin/
```

### Windows (Pre-built Binary)

you can download the pre-build binary from the [Releases](https://github.com/sreedevk/deduplicator/releases) page.
download the `deduplicator-x86_64-pc-windows-msvc.zip` zip file for windows. unzip the `zip`  file & move the `deduplicator.exe` to a location in the PATH system environment variable.

Note: If you Run into an msvc error, please install MSCV from [here](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)

## Performance

Deduplicator uses size comparison and gxhash (a non non-cryptographic hashing algorithm) to quickly scan through large number of files to find duplicates. its also highly parallel (uses rayon and dashmap). 

## Screenshots
![](https://user-images.githubusercontent.com/36154121/213618143-e5182e39-731e-4817-87dd-1a6a0f38a449.gif)

## Roadmap
    - Tree format output for duplicate file listing
    - GUI
    - Packages for different operating system repositories (currently only installable via cargo) 

## v0.3 checklist
- [x] parallelization
    - [x] (scanning) + (processing sw & processing hw & formatting & printing)
    - [ ] (scanning + processing sw + processing hw) & formatting & printing
    - [ ] scanning + processing sw + processing hw + formatting + printing
- [x] reduce cloning values on the heap
- [x] add a partial hashing mode (--strict)
- [ ] max file path size should use the last set of duplicates
- [-] add unit tests
- [x] add silent mode
- [ ] restore json output
- [ ] update documentation
- [x] remove color output
- [x] progress bar improvements
    - [x] use progress bar groups
- [ ] fix memory leak on very large filesystems
    - [ ] maybe use a bloom filter
    - [ ] reduce FileInfo size
