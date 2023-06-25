# bexpand

Bash-style brace expansion in Rust.

## Functionality

* Plain strings are just plain strings: `abcd`
* Special characters need to be escaped with a preceding `\` to be considered as regular characters: `ab\{cd`
  * Outside of a list, special characters are `{}\`.
  * Inside of a list, special characters are `,{}\`.
  * Inside of a sequence, special characters are `,.{}\`.
* Any character preceded by a `\` will be taken literally, even if it's redundant.
  * `\n`, for example, represents `n`, not a newline character.
* A List is a brace-enclosed comma-separated list of Expressions: `{a,b,c}`, `{a,,c}`, `{}`, `{,}`
  * An empty List is still a List.
  * Lists may have empty or duplicate fields.
  * Lists produce all the values of their contained Expressions.
* A Sequence is either a Numeric Sequence or a Character Sequence.
* A Numeric sequence is in the form `{<spec><start>..<end>[..<stride>]}`
  * `<spec>` is a possibly-empty set of format specifier characters:
    * `=` means to expand each item with leading zeroes to the width of the
      longest character width of `<start>` and `<end>`.
  * `<start>` and `<end>` are signed 64-bit integers.  If `<end>` is less than
    `<start>`, the sequence will count downwards.
  * `<stride>` is an optional non-negative increment number, to count by
    increments of more than 1.
    * The default `<stride>` is `1`.
    * A zero `<stride>` is always normalized to `1` to prevent infinite looping.
    * A stride may cause the endpoint to be skipped, and even the numeric size
      limit to be hit without error.
      * `'{9223372036854775806..9223372036854775807..1000}'` just produces
        `9223372036854775806`, not an error.
* A Character sequence is in the form `{<start>..<end>[..<stride>]}`
  * `<start>` and `<end>` are unicode characters to produce codepoints for
    in order.  If `<end>` is less than `<start>`, the sequence will cycle
    downwards.  If this range would end up producing a surrogate codepoint, an
    error is given for each instead.
    * An error does not terminate iteration.  If an error is returned, following
      iterations that move out of the surrogate range may still produce good
      values.
      * This could be used at some point to allow optional replacement
        characters, but I don't see a value in that over just throwing an error
        at this time.
  * `<stride>` is an optional non-negative increment number, to count by
    increments of more than 1.
    * The default `<stride>` is `1`.
    * A zero `<stride>` is always normalized to `1` to prevent infinite looping.
    * A stride may cause the endpoint to be skipped, and even the numeric size
      limit to be hit without error.
      * `'{a..z..1114111}'` just produces `a`, not an error
* An Expression contains a sequence of Plain strings, Lists, and Sequences.
  * An Expression produces the cartesian product of all its items:
    `{a,b}c{d,e}f{g..i}` produces
    `["acdfg","acdfh","acdfi","acefg","acefh","acefi","bcdfg","bcdfh","bcdfi","bcefg","bcefh","bcefi"]`
  * Expression order is produced in lexicographic order, keyed by the index of
    each sub-expression.
* Expressions and Lists may nest arbitrarily.
  * `'{a,{b,,c{\,..\.}}{f..d..2}}'` produces `["a","bf","bd","f","d","c,f","c,d","c-f","c-d","c.f","c.d"]`

## Differences from Bash

This does not 100% conform to Bash's style in the following ways:

* There are patterns considered ill-formed and will throw an error in bexpand.
  It will not try to truck along if a bad pattern is found.
* Braces are special characters and are not allowed without either being
  correctly formed or being escaped.  In Bash `a{b,c}d}e` expands to
  `abd}e acd}e` and `a{b{c,d}e` expands to `a{bce a{bde`. In bexpand, both are
  errors.
* Empty and single-component lists are considered acceptable in bexpand.  In
  Bash, `a{}b` and `a{b}c` are both literally repeated by the shell.  In
  bexpand, these expand to `ab` and `abc`.
* bexpand allows character sequences to iterate any valid unicode codepoints.
  `{🥰..🥴..2}` is a valid character sequence, as is `{\{..\.}`, and `{9..A}`.
  Technically, `{\0..\9}` is valid as well, and will be treated as a character
  sequence, though it expands to the exact same thing as a numeric sequence of
  the same form.  Anything that would generate an illegal unicode codepoint will
  generate an error.
* The width specifier is done with an equal sign at the beginning of the
  opening brace instead, so in Bash, `{001..100}` is instead done in bexpand as
  `{=1..100}`.  This is to allow things like `{=-5..10}`, which is impossible to
  express in Bash.

## License

Copyright 2023 Taylor Richberger

Published under the terms of the Mozilla Public License Version 2.0.
