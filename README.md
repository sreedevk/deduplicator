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

Deduplicator uses size comparison and fxhash (a non non-cryptographic hashing algo) to quickly scan through large number of files to find duplicates. its also highly parallel (uses rayon and dashmap). I haven't uploaded the benchmarks yet, but I was able to scan through 120GB of files (Videos, PDFs, Images) in ~300ms.

## Screenshots

![](https://user-images.githubusercontent.com/36154121/213618143-e5182e39-731e-4817-87dd-1a6a0f38a449.gif)
