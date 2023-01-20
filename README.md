<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

<p align="center">
NOTE: This project is still being developed. At the moment, as shown in the screenshot below, deduplicator is able to scan through and list duplicates with and without caching. Contributions are welcome.
</p>

<h2 align="center">Usage</h2>

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

<h2 align="center">Installation</h2>

<p align="center">Currently, deduplicator is only installable via rust's cargo package manager</p>

```
cargo install deduplicator
```
<p align="center">
  note that if you use a version manager to install rust (like asdf), you need to reshim (`asdf reshim rust`).
</p>

<h2 align="center">Performance</h2>

<p align="center">
  Deduplicator uses fxhash (a non-cryptographic hashing algorithm) which is extremely fast. As a result, deduplicator is able to process huge amounts of data in a <del>couple of seconds.</del> few milliseconds.</p>

<p align="center">
  <del>While testing, Deduplicator was able to go through 8.6GB of pdf files and detect duplicates in 2.9 seconds</del>
  As of version 0.1.1, on testing locally, deduplicator was able to process and find duplicates in 120GB of files (Videos, PDFs, Images) in ~300ms
</p>

<h2 align="center">Screenshots</h2>

<img src="https://user-images.githubusercontent.com/36154121/213618143-e5182e39-731e-4817-87dd-1a6a0f38a449.gif" />
