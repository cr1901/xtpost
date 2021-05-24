# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project does _not_ strictly adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Major version changes are reserved for code restructuring changes. Minor
version changes are reserved for new features. Patch level changes are reserved
for bug fixes in between minor versions.

# `xtpost` Retro Computer Server Command-Line Driver

## [Unreleased]

## v0.1.0 <!-- - 2021-05-22 -->
Initial release.

- Client program uses [reqwest] to interface to [reenigne](https://github.com/reenigne)'s
  [XT server](http://www.reenigne.org/xtserver/). Serial output (`int 0x63`,
  `int 0x64`, `int 0x65`), image capture (`int 0x60`), file capture
  (`int 0x66`), and audio capture (`int 0x61`, `int 0x62`) are all supported.
- Use [Github Actions](https://github.com/cr1901/xtpost/actions) and
  [Github Releases](https://github.com/cr1901/xtpost/releases) to support
  x86_64 Windows, MacOS, and Linux binaries.

[reqwest]: https://github.com/seanmonstar/reqwest

[Unreleased]: https://github.com/cr1901/xtpost/compare/v0.1.0...HEAD
