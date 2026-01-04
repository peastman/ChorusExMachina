# Chorus Ex Machina

Chorus Ex Machina is an open source, physically modelled chorus synthesizer.  It can sing
realistic sounding text in a variety of languages.  [Here is an example](https://on.soundcloud.com/9HPY867cTWf5CY4y7)
of music created with it, so you can hear what it sounds like.

The sound produced by Chorus Ex Machina is completely dry.  It models only the singers, not
the room they are in.  To get a realistic sound, it is essential that you add an appropriate
reverb.

### Installing and Using

Chorus Ex Machina can be used as a VST3, CLAP, or AUv2 plugin.  [The Releases page](https://github.com/peastman/ChorusExMachina/releases)
has compiled versions for Windows, Linux, and macOS.  If instead you want to build it from
source, first install the [Rust compiler](https://www.rust-lang.org/).  To build the VST3 and CLAP
plugins, execute the following command from this directory.

```
cargo xtask bundle chorus_ex_machina --release
```

To build the AUv2 plugin, first build the CLAP plugin then follow the instructions in
the `au` subdirectory.

On macOS, you may find it is necessary to compile the plugin yourself.  By default, Apple
blocks all programs from running unless they are digitally signed by a developer who pays
$99/year for an account, which I choose not to do.  There are workarounds which can allow
the precompiled versions to work, but those workarounds have gotten steadily more difficult
with time.  Compiling it yourself avoids this problem.

For instructions on how to use the plugin, see [the documentation](plugin/src/help.md),
which is also available in the plugin's user interface.

### Keyboard Input Issues

Some DAWs intercept some or all keystrokes and interpret them as commands to the DAW itself
rather than sending them to the plugin's editor.  This interferes with typing text into the
Chorus Ex Machina editor window.  There are workarounds to address this problem.

**Reaper**: In the FX window, click the "+" button at the top of the window.  Select
"Send all keyboard input to plug-in".

**Ableton**: Create a text file called `Options.txt` and place it in the `Preferences`
directory.  It should contain the line

```
-_EnsureKeyMessagesForPlugins
```

See the [Ableton Knowledge Base](https://help.ableton.com/hc/en-us/articles/6003224107292-Options-txt-file)
for more details.