# Shitty Scheme Interpreter #

Made this to try different continuation implementations,
hope I get to do that! :)

Lots of function semantics, among other things, taken from racket docs

Implementation hints taken from
[minilisp](https://github.com/rui314/minilisp). Thanks rui314!

TODO:
* Get `cargo fmt` working?
* All the TODOs in the code LOL
* String interning of symbols!!!
* Replace usage of `panic!(...)` and other possibly panicking things with proper error handling. Things like `if idx_valid { vec[idx] }` can be kept.
* a lot of pub functions should probably just be `pub(crate)`?
* Symbol interning with some rust string-intern crate like Lasso.

## Continuations ##

What is a continuation, you ask? Basically, a really high-powered
control-flow mechanism. Its reifed stack-frames. Captured into a variable,
used like a function.

I implement a simple type of continuation in
[SinScheme](https://github.com/sinistersnare/SinScheme).

It's a bit more complicated for this interpreter.
Im hoping to try different implementations of continuations here.
From TODO: put papers here.

## Usage ##

`cargo run` will run a REPL! Quit with Ctrl-C, Ctrl-D doesnt work.

`cargo run -- file.scm` will run `file.scm`.

LICENSE:

MIT LICENSE. I'm happy if you just say 'hey Sinistersnare nice code!'.
But you don't have to.
