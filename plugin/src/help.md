# Entering Text

A phrase consists of a series of syllables, separated by spaces, written in [X-SAMPA notation](https://en.wikipedia.org/wiki/X-SAMPA).
For example, to sing the words "Happy birthday to you," enter the phrase `h{ pe b3T dEj tu ju`.  Each successive note is sung on
the next syllable.  When it reaches the end of the current phrase, it loops back to the start again.

You can enter up to 128 phrases.  To select the phrase to sing, click on it in the table.  To automate it in a DAW, set the parameter
"Selected Phrase" to the index of the phrase to sing.

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
| i | b**e** | Often changed to `e` when singing |
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
  "eterna" is often sung as `E tE nA`.

# Playing

Chorus Ex Machina is a monophonic instrument: each instance plays only one note at a time.  To
create splits within a section use multiple tracks, each with its own instance of the plugin.  If you
press a new note before releasing the previous one, it is played legato, smoothly blending between
them.  Legato transitions are only possible between vowels, however.  If there are any consonants
in between, they necessarily create a break in the sound.

There are several parameters you can automate in a DAW to control the performance.

- **Selected Phrase**.  The index of the phrase to sing.
- **Dynamics**.  How loud to sing.  This is not simply a volume control.  Voices sound different
  depending on how loudly they are singing.
- **Vibrato**.  The amount of vibrato in the voices.  For choruses, it is best to stick to values
  near the middle of the range.  Solo voices can use more extreme values to add expression.
- **Intensity**.  How relaxed or intense the voices sound.
- **Stereo Width**.  How widely the singers are spread out in space.
