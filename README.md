# Chorus Ex Machina

Chorus Ex Machina is an open source, physically modelled chorus synthesizer.  It can sing
realistic sounding text in a variety of languages.  The current code is still an early
version, but progressing rapidly.

Chorus Ex Machina can be packaged as a VST3, CLAP, or AUv2 plugin.  To build it, first
install the [Rust compiler](https://www.rust-lang.org/).  To build the VST3 and CLAP
plugins, execute the following command from this directory.

```
cargo xtask bundle chorus_ex_machina --release
```

To build the AUv2 plugin, first build the CLAP plugin then follow the instructions in
the `au` subdirectory.