# Shitty Scheme Interpreter #

Made this to try different continuation implementations,
hope I get to do that! :)

I am using semantics I defined [here](https://github.com/sinistersnare/aams/blob/master/latex/formalism.pdf).
There is also an interpreter in Racket in that repo, among other things.
This one is very...typed... compared to that one, wanted to see
how well my formalization worked type-wise.

TODO:
* All TODOs obviously
* Is it possible to use &str instead of so many owned values? I wonder how hard that would be...
	* Would require `combine` to return `&str` instead of `String`. Need to look into that!

## Continuations ##

What is a continuation, you ask? Basically, a really high-powered
control-flow mechanism. Its reifed stack-frames. Captured into a variable,
used like a function.

I implement a simple type of continuation in
[SinScheme](https://github.com/sinistersnare/SinScheme).
But thats a compiler, and by translating to CPS

It's a bit more complicated for this interpreter.
Im hoping to implement different types of continuations here.
The most basic is 'one-shot' continuations, which are pretty inefficient.
I want to get to implementing delimited continuations if I can.

TODO: put papers here.

Also I will be writing a blog post on continuations soon, so check my blog out!

## Usage ##

`cargo run` will run a REPL! Quit with Ctrl-C, Ctrl-D is just end-of-line.

`cargo run -- file.scm` will run `file.scm`.

## LICENSE: ##

MIT LICENSE. I'm happy if you just say 'hey Sinistersnare nice code!'.
But you don't have to.
