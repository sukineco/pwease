# pwease

Simple utility for user and group substitution.

## Why

Because neither `doas` nor `sudo` can subtitute supplementary groups (maybe
`sudo` could but it's too bloated for me anyways). And also `pwease` has more
reasonable defaults (envrionment inheritance, `permit nopass :wheel`).

## Configuration

As of right now there is no way to configure `pwease` but it might change in
future!

## Installation

Note that second step requires running as root.

```sh
$ make
# make install
```
