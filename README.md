# bexpand

Bash-style brace expansion in Rust.

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

# License

Copyright 2023 Taylor Richberger

Published under the terms of the Mozilla Public License Version 2.0.
