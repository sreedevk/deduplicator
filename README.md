<h1 align="center">Deduplicator</h1>

<p align="center">
  Find, Sort, Filter & Delete duplicate files 
</p>

NOTE: This project is still being developed. At the moment, as shown in the screenshot below, deduplicator is able to scan through and list duplicates with and without caching. Contributions are welcome.

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
<h2 align="center">Performance</h2>

<p align="center">
  Deduplicator uses fxhash (a non-cryptographic hashing algorithm) which is extremely fast. As a result, deduplicator is able to process huge amounts of data in a couple of seconds.</p>

  <p align="center">
    While testing, Deduplicator was able to go through 8.6GB of pdf files and detect duplicates in 2.9 seconds
  </p>
<h2 align="center">Screenshots</h2>

![_039](https://user-images.githubusercontent.com/36154121/210031222-d8b79143-5a1e-47ca-926e-8855d5bbab60.png)
