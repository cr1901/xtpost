# `xtpost`

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
