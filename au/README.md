# Chorus Ex Machina Audio Unit Plugin

This directory uses [clap-wrapper](https://github.com/free-audio/clap-wrapper) to create an
Audio Unit (AUv2) plugin by wrapping the CLAP plugin.  It uses CMake for the build script.
Make sure you have already built the CLAP plugin, then execute the following commands from this
directory.

```
mkdir build
cd build
cmake ..
make
```

To install the plugin, copy `Chorus Ex Machina.component` from `build` to `~/Library/Audio/Plug-Ins/Components`.