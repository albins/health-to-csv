# What is this?

This is a small program to turn your Apple Health exported data into a CSV in
the simplest possible way. It reads the data straight out of the zip file, but
beware: it has to decompress it into RAM, and it can get large.

It's worth pointing out that it *only* exports `Record`s so far.

# Usage

```
$ RUST_LOG=info cargo run --release -- export.zip > results.csv

```

The release build is really recommended, as the debug build is significantly
slower. Output (the CSV) is written to stdout, which may not be ideal.
