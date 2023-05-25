# Raw Timestamper
External dependencies: [gexiv2](https://wiki.gnome.org/Projects/gexiv2) and [Exiv2](http://www.exiv2.org/)

```
RAW timestamper: adds a timestamp to the beginning of RAW (and their XMP sidecar) files

Usage: rrn [OPTIONS] [FILES]...

Arguments:
  [FILES]...
          List of RAW files to rename. Any XMP sidecars present will be renamed alongside their corresponding RAWs

Options:
  -n, --dry-run
          Perform a trial run with no changes made

  -d, --datestamp
          Use datestamps (YYYY-MM-DD) without the time of day information

  -q, --quiet
          

  -b, --benchmark
          

  -h, --help
          Print help (see a summary with '-h')
```
