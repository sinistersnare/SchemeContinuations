# Shitty Scheme Interpreter #

Made this to try different coroutine implementations,
hope I get to do that! :)

Lots of function semantics, among other things, taken from racket docs

Implementation hints taken from
[minilisp](https://github.com/rui314/minilisp). Thanks rui314!

TODO:
* Replace usage of `panic!(...)` and other possibly panicking things with proper error handling. Things like `if idx_valid { vec[idx] }` can be kept.
* a lot of pub functions should probably just be `pub(crate)`?

LICENSE:

MIT LICENSE. I'm happy if you just say 'hey Sinistersnare nice code!'.
But you don't have to.
