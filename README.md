<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

## Usage

```bash
Usage: deduplicator [OPTIONS]

Options:
  -t, --types <TYPES>      Filetypes to deduplicate (default = all)
      --dir <DIR>          Run Deduplicator on dir different from pwd
  -i, --interactive        Delete files interactively
  -m, --minsize <MINSIZE>  Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T). [default = 0]
  -h, --help               Print help information
  -V, --version            Print version information
```

## Installation

### Cargo Install

#### Stable

```bash
$ cargo install deduplicator
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

Deduplicator uses size comparison and fxhash (a non non-cryptographic hashing algo) to quickly scan through large number of files to find duplicates. its also highly parallel (uses rayon and dashmap). I was able to scan through 120GB of files (Videos, PDFs, Images) in ~300ms. checkout the benchmarks

## benchmarks

| Command | Dirsize | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|:---|---:|---:|---:|---:|
| `deduplicator --dir ~/Data/tmp` | (~120G) | 27.5 ± 1.0 | 26.0 | 32.1 | 1.70 ± 0.09 |
| `deduplicator --dir ~/Data/books` | (~8.6G) | 21.8 ± 0.7 | 20.5 | 24.4 | 1.35 ± 0.07 |
| `deduplicator --dir ~/Data/books --minsize 10M` | (~8.6G) | 16.1 ± 0.6 | 14.9 | 18.8 | 1.00 |
| `deduplicator --dir ~/Data/ --types pdf,jpg,png,jpeg` | (~290G) | 1857.4 ± 24.5 | 1817.0 | 1895.5 | 115.07 ± 4.64 |

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
