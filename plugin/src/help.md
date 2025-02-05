# Entering Text

A phrase consists of a series of syllables, separated by spaces, written in [X-SAMPA notation](https://en.wikipedia.org/wiki/X-SAMPA).
For example, to sing the words "Happy birthday to you," enter the phrase `h{ pi b3T dEj tu ju`.  Each successive note is sung on
the next syllable.  When it reaches the end of the current phrase, it loops back to the start again.

You can enter up to 128 phrases.  To select the phrase to sing, click on it in the table.  To automate it in a DAW, set the parameter
"Selected Phrase" to the index of the phrase to sing.  You can use the "Advance Syllable" parameter to temporarily disable advancing
to the next syllable.  When this option is off, every note continues to use the same syllable until it is turned back on.  This is
useful when a phrase is sung repeatedly, but the number of notes each syllable is used for changes between repetitions.

A syllable consists of

- Zero or more initial consonants
- One or more vowels
- Zero or more final consonants

When a syllable contains multiple vowels, one of them is the main vowel that is held for the duration of the note.  The others are
transient vowels that are touched only briefly at the beginning or end.  Chorus Ex Machina tries to make a reasonable guess about
which one should be the main vowel.  If it guesses wrong, you can indicate the main vowel by adding a `-` immediately after it.
Compare `A-i` and `Ai-`.

The following vowels are supported.

| Symbol | Pronunciation | Notes |
| --- | --- | --- |
| A | f**a**ther | |
| E | m**e**t | |
| I | k**i**t | |
| N | thi**ng** | |
| O | **o**ff | |
| U | f**oo**t | |
| V | str**u**t | |
| Y | h**ü**bsch (German) | |
| a | d**a**me (French) | |
| e | beaut**é** (French) | |
| i | b**e** | Sometimes changed to `y` when singing |
| l | **l**ot | |
| m | hi**m** | |
| n | **n**ap | |
| o | v**eau** (French) | |
| u | b**oo**t | |
| y | **ü**ber (German) | |
| { | c**a**t | |
| @ | **a**ren**a** | |
| & | sk**ö**rd (Swedish) | |
| 2 | d**eu**x (French) | |
| 3 | n**ur**se | See note on the letter R below |
| 9 | n**eu**f (French) | |

The following consonants are supported.

| Symbol | Pronunciation | Notes |
| --- | --- | --- |
| C | i**ch** (German) | |
| D | **th**is | |
| S | **sh**ip | |
| T | **th**in | |
| Z | vi**si**on | |
| b | **b**ed | |
| d | **d**og | |
| f | **f**ive | |
| g | **g**ame | |
| h | **h**ouse | |
| j | **y**ou | |
| k | **c**ut | |
| p | **p**it | |
| r | pe**rr**o (Spanish) | See note on the letter R below |
| s | **s**eem | |
| t | **t**in | |
| v | **v**est | |
| w | **w**est | |
| x | lo**ch** | |
| z | **z**oo | |
| 4 | ca**r**o (Spanish) | See note on the letter R below |
| dZ | **j**eans | |
| tS | **ch**eap | |

**Note on the letter R**

Several different sounds can be used for the letter R, depending on the language and context.

- The "flipped R" `4` is the most commonly used R sound in choral singing.  When a spoken word uses
  a different sound, it often is changed to `4` when sung.
- The "rolled R" `r` is common in Spanish and Italian.  It often is changed to `4` in choral singing.
- For the non-rolled R common in English, use the vowel `3`.
- In spoken French, R is often pronounced as the "uvular trill" `R\`.  This sound is not commonly
  used in choral singing, and is not supported by Chorus Ex Machina.  It usually is changed to `4`,
  or occasionally to `r`.
- When R appears as a final consonant, it sometimes is omitted.  For example, the Latin word
  "eterna" is often sung as `E tE nA`.  Alternatively, it may be replaced by a neutral vowel such
  as `@` or `9`.

**Choosing vowels**

Your choices of what vowels to use are important for creating a natural sound.  The vowels used in
singing, and especially in choral singing, are often different from the ones used when speaking.
There are many vowels that sound very similar to each other.  Some examples are

- `A` and `a`
- `i` and `y`
- `@`, `V`, and `9`

Which one sounds best depends not only on the word being sung but also on the pitch, the voice
part, the surrounding words, and the style of music.  Consider the English word "I".  It can be
pronounced in many subtly different ways: `Ai`, `Ae`, `AI`, `a-I`, etc.  If a note sounds harsh
or unnatural, experiment with different vowels to find the combination that sounds best.

**Placing consonants**

Consonants can appear both at the beginning and the end of a syllable.  Many consonants sound slightly
different depending on the position.  Initial consonants tend to be slightly louder and longer than
final consonants.

When a consonant appears between two vowels that are sung legato (see below), you can choose which
position to put it in.  For example, the word "open" can be pronounced either as `op En` or `o pEn`.
Each one involves the same series of sounds, but the "p" is pronounced differently.

# Playing

Chorus Ex Machina is a monophonic instrument: each instance plays only one note at a time.  To
create splits within a section use multiple tracks, each with its own instance of the plugin.  If you
press a new note before releasing the previous one, it is played legato, smoothly blending between
them.  Smooth transitions are only possible between vowels, however.  If there are any consonants
in between, they necessarily create a break in the sound, but it will shorten the consonants to
minimize the gap.

You can optionally add an accent to the beginning of each separated (non-legato) note.  When this
option is enabled, the strength of the accent is determined by the key velocity.

The Vowel Delay parameter is an important tool for creating accurate timing, especially in fast
passages.  When a syllable begins with one or more consonants, the consonants are normally sung
before the beat, and the vowel begins right on the beat.  This requires the start of the note to
be shifted earlier.  How far it must be shifted depends on the number and type of consonants,
possibly including final consonants from the previous note.  Adjusting note start times by hand is
slow and imprecise.

Instead you can tell it to delay the start of the first vowel by up to 200 ms.  This allows the
vowel to always follow the start of the note by exactly the same amount, regardless of what
consonants precede it.  When singing legato, the end of the previous note is shifted by the same
amount to avoid creating a break in the sound.  This gives precise timing with no extra effort.
If you need to synchronize the chorus with other instruments, you can adjust the MIDI offset in
your DAW to shift the start of the notes earlier by the same amount as the delay, so vowels begin
exactly on the beat.

There are several parameters you can automate in a DAW to control the performance.

- **Selected Phrase**.  The index of the phrase to sing.
- **Dynamics**.  How loud to sing.  This is not simply a volume control.  Voices sound different
  depending on how loudly they are singing.
- **Vibrato**.  The amount of vibrato in the voices.  For choruses, it is best to stick to values
  near the middle of the range.  Solo voices can use more extreme values to add expression.
- **Intensity**.  How relaxed or intense the voices sound.
- **Brightness**.  The overall vowel color.  Reduce this to get a darker sound.
- **Consonant Volume**.  How loud consonants are relative to vowels.  This can be automated for
  fine control over individual consonants.
- **Attack Rate**.  How quickly notes reach full volume.
- **Release Rate**.  How quickly the sound stops at the end of a note.
- **Stereo Width**.  How widely the singers are spread out in space.
- **Vowel Delay**.  The delay in milliseconds from the start of each note to the beginning of the
  first vowel.
- **Accent**.  Whether to add an accent to each note based on its velocity.
- **Advance Syllable**.  Whether to advance to the next syllable in the phrase for the next note.