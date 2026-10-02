# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-10-02

Huge unatomized commit, yeah, bad, I know, sorry, won't ever happen again I
hope.

### Added

- Root substitution (`--root`).
- Running command as a login shell (`--login`).
- Home substitution (`--home`).
- Clearing groups (`--clear-groups`).
- Flag explicitly disabling user substitution (`--same-user`).

## Changed

- `--groups` flag now add additional supplementary groups on top of the *base
  set*. *Base set* is
  - groups of the parent process if `--inherit-groups` is provided;
  - empty if `--clear-groups` is provided;
  - groups of the target user according to PAM otherwise.
- Running without a command now starts a shell.
- User substitution now always happens unless `--same-user` is provided.
- Migrated from `Makefile` to `install.sh`.
- Full README rewrite.

## Removed

- `-v` shorthand for `--version`.

[Unreleased]: https://github.com/sukineco/pwease/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/sukineco/pwease/releases/tag/v1.0.0
