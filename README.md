# `xtpost`
[![Build Status](https://github.com/cr1901/xtpost/actions/workflows/ci.yml/badge.svg)](https://github.com/cr1901/xtpost/actions)
[![GitHub release](https://img.shields.io/github/release/cr1901/xtpost.svg)](https://github.com/cr1901/xtpost/releases)
[![GitHub license](https://img.shields.io/github/license/cr1901/xtpost.svg)](https://github.com/cr1901/xtpost/blob/master/LICENSE.md)
[![Last commit](https://img.shields.io/github/last-commit/cr1901/xtpost.svg)](https://GitHub.com/cr1901/xtpost/commit/)
[![Contact Me](https://img.shields.io/twitter/follow/cr1901.svg?label=Contact%20Me&&style=social)](https://twitter.com/cr1901)

`xtpost` is a command-line application designed to interface to [reenigne](https://github.com/reenigne)'s
[XT server](http://www.reenigne.org/xtserver/).

## Usage

Type `xtpost --help` for help, but the general usage is:

```
xtpost run [binfile]
```

`xtpost` uses a `settings.json` file for your email and other configuration.
`settings.json`, as well as data such as composite video/audio capture are
stored in system-specific directories. These directories can be listed with:

```
xtpost cfg -d
```

A default `settings.json` will be created for you if it doesn't exist- _I
recommend adding an email address_.

## Releases

Currently releases are made for the following Rust targets:

* `x86_64-apple-darwin`
* `x86_64-unknown-linux-gnu`
* `x86_64-pc-windows-gnu`

In the future, releases will be made for the following Rust targets (there are
CI [problems](https://github.com/cr1901/xtpost/runs/2651899502?check_suite_focus=true#step:5:371)
I'm not sure how to deal with right now):
* `aarch64-unknown-linux-gnu`
* `armv7-unknown-linux-gnueabihf`

Once the above issues are resolved, if you want a new target added to CI, ask
me and I'll make a point release. In the meantime, you can clone this repo
and run `cargo build --release` to get a binary if you have a Rust toolchain
installed.
