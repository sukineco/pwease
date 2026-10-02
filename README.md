# pwease

Command line utility for <insert thing name> substitution.

## Rationale

There is a huge ton of different utilities for different kinds of substitution,
and sometimes you even need to combine them. What if you want to just substitute
groups for a new process (kepping the same user)? Or *properly* (with working
/dev, /sys and /proc) chroot into somewhere? What if you also wanna login in
chrooted environment? And so on. These seemingly unrelated problems in practice
could arise together and combining different substitution programs is not
convenient enough at least for me.

## Abstract

As of right now, `pwease` can:

- substitute user (like `sudo` or `doas`);
- substitute groups;
- change root (binding /dev, /sys and /proc);
- run command as a login shell.

More features are probably coming.

## Defaults

- Envrionment is kept. Use `--no-keep-env` to clear it.
- `$HOME` is kept. Use `--home` to also substitute it.
