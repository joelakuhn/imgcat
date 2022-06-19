# imgcat

`imgcat` is a simple utility that prints images to the terminal using lower half block characters.

## Usage

```
Usage:
  target/release/imgcat [OPTIONS] [FILES ...]

Positional arguments:
  files                 Image files to process

Optional arguments:
  -h,--help             Show this help message and exit
  -w,--width WIDTH      Specify width
  -h,--height HEIGHT    Specify height
  -t,--triangle         Use triangle algorithm (default)
  -n,--nearest          Use nearest neighbor algorithm
  -l,--lanczos          Use lanczos3 algorithm
```
