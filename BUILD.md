# Building

```sh
$ ./install.sh
```

You can change output location by exporting `$PREFIX` and can change target by
exporting `$TARGET`.

```sh
$ mkdir bin
$ export PREFIX=.
$ export TARGET=x86_64-unknown-linux-musl # The default value
$ ./install.sh
```
