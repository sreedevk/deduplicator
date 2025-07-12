<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

## Usage

```bash
Usage: deduplicator [OPTIONS] [scan_dir_path]

Arguments:
  [scan_dir_path]  Run Deduplicator on dir different from pwd (e.g., ~/Pictures )

Options:
  -t, --types <TYPES>          Filetypes to deduplicate [default = all]
  -i, --interactive            Delete files interactively
  -s, --min-size <MIN_SIZE>    Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T) [default: 1b]
  -d, --max-depth <MAX_DEPTH>  Max Depth to scan while looking for duplicates
      --min-depth <MIN_DEPTH>  Min Depth to scan while looking for duplicates
  -f, --follow-links           Follow links while scanning directories
  -h, --help                   Print help information
  -V, --version                Print version information
      --json                    
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
> please install version `0.2.1`  if you are unable to install `0.2.2`

```bash
$ RUSTFLAGS="-C target-cpu=native" cargo install deduplicator
```

> [!]

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

Deduplicator uses size comparison and fxhash (a non non-cryptographic hashing algo) to quickly scan through large number of files to find duplicates. its also highly parallel (uses rayon and dashmap). I was able to scan through 120GB of files (Videos, PDFs, Images) in ~300ms. checkout the benchmarks

## benchmarks

| Command | Dirsize | Filecount | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|:---|---:|---:|---:|---:|---:|
| `deduplicator ~/Data/tmp` | (~120G) | 721 files | 33.5 ± 28.6 | 25.3 | 151.5 | 1.87 ± 1.60 |
| `deduplicator ~/Data/books` | (~8.6G) | 1419 files | 24.5 ± 1.0 | 22.9 | 28.1 | 1.37 ± 0.08 |
| `deduplicator ~/Data/books --min-size 10M` | (~8.6G) | 1419 files | 17.9 ± 0.7 | 16.8 | 20.0 | 1.00 |
| `deduplicator ~/Data/ --types pdf,jpg,png,jpeg` | (~290G) | 104222 files | 1207.2 ± 37.0 | 1172.2 | 1287.7 | 67.27 ± 3.33 |

* The last entry is lower because of the number of files deduplicator had to go through (~660895 Files). The average size of the files rarely affect the performance of deduplicator.

These benchmarks were run using [hyperfine](https://github.com/sharkdp/hyperfine). Here are the specs of the machine used to benchmark deduplicator:

```
OS: Arch Linux x86_64 
Host: Precision 5540
Kernel: 5.15.89-1-lts 
Uptime: 4 hours, 44 mins 
Shell: zsh 5.9                        
Terminal: kitty 
CPU: Intel i9-9880H (16) @ 4.800GHz 
GPU: NVIDIA Quadro T2000 Mobile / Max-Q 
GPU: Intel CoffeeLake-H GT2 [UHD Graphics 630] 
Memory: 31731MiB (~32GiB)
```

## Screenshots

![](https://user-images.githubusercontent.com/36154121/213618143-e5182e39-731e-4817-87dd-1a6a0f38a449.gif)

## Roadmap
    - Tree format output for duplicate file listing
    - GUI
    - Packages for different operating system repositories (currently only installable via cargo) 

## v0.3 checklist
- [x] parallelization of scanning, processing and formatting
- [x] reduce cloning values on the heap
- [ ] add a partial hashing mode
- [ ] add an option to use a bloom filter for very large filesystems
- [ ] max file path size should use the last set of duplicates
