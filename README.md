# Chorus Ex Machina

Chorus Ex Machina is an open source, physically modelled chorus synthesizer.  It can sing
realistic sounding text in a variety of languages.  [Here is an example](https://on.soundcloud.com/9HPY867cTWf5CY4y7)
of music created with it, so you can hear what it sounds like.

Chorus Ex Machina can be used as a VST3, CLAP, or AUv2 plugin.  [The Releases page](https://github.com/peastman/ChorusExMachina/releases)
has compiled versions for Windows, Linux, and macOS.  If instead you want to build it from
source, first install the [Rust compiler](https://www.rust-lang.org/).  To build the VST3 and CLAP
plugins, execute the following command from this directory.

```
cargo xtask bundle chorus_ex_machina --release
```

To build the AUv2 plugin, first build the CLAP plugin then follow the instructions in
the `au` subdirectory.

For instructions on how to use the plugin, see [the documentation](plugin/src/help.md),
which is also available in the plugin's user interface.