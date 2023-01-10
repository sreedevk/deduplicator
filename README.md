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
  -t, --types <TYPES>  Filetypes to deduplicate (default = all)
      --dir <DIR>      Run Deduplicator on dir different from pwd
  -n, --nocache        Don't use cache for indexing files (default = true)
  -h, --help           Print help information
  -V, --version        Print version information
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
  Deduplicator uses fxhash (a non-cryptographic hashing algorithm) which is extremely fast. As a result, deduplicator is able to process huge amounts of data in a couple of seconds.</p>

  <p align="center">
    While testing, Deduplicator was able to go through 8.6GB of pdf files and detect duplicates in 2.9 seconds
  </p>
<h2 align="center">Screenshots</h2>

<p align="center">
  <img align="center" src="https://user-images.githubusercontent.com/36154121/211458077-90092aa3-496c-492f-a061-618059890d5f.png" width="500" height="400" />
</p>

