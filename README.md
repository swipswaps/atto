# Atto

Atto is an insanely simple functional programming language.

It features a syntax driven entirely by polish notation and no delimiters to speak of (it ignores all non-separating whitespace).
What do you get for this simplicity? Well... an insanely simple language with a ~200 line self-hosted interpreter.

Despite these design limitations, it's actually possible to write quite pleasing code in Atto.
That, combined with the extraordinarily extendable syntax (you can define your own operators, or overload those defined in the `core` library) make it
ideal for solving a whole class of programming problems that are normally awkward to solve in more imperative languages.

## Design

Atto's design is stupidly simple. There are two kinds of structure:

Functions: `fn <name> [args] is <expr>`

Expressions: `<literal> [expr]`

That's it. Expressions, function calls, literals and operations are all considered to be the same thing.

I leave you with a quick factorial calculation example demonstrating the compact expressiveness of Atto at work.

```
fn f n is
  if = n 0
    1
  * n f - n 1
```

Yes, that's it.

## Atto Interpreter Written In Atto

In `examples/self-hosted.at`, I've written a fully-functioning REPL-based interpreter for Atto.
It supports function declaration, function calling, and all of the evaluation operators that Atto does, including I/O.
It has a minor issues, such as behaving unpredictably with invalid input. However, it should be able to successfully run any valid Atto program (provided your stack is big enough).

Which reminds me: I need to use a non-recursive interpretation algorithm in the Rust interpreter. Also, tail-call optimisation would be nice.

## Core Library

Atto comes with a `core` library. It provides a series of non-intrinsic functions and utilities that are themselves written in Atto.
In addition, it provides all of the operators common to Atto usage.
The Atto interpreter implicitly inserts the `core` library above whatever you run, similar in nature to C's `#include`.

- `# x y`: Ignore the first value, evaluate to only the second (useful for comments)
- `@ x y`: Ignore the second value, evaluate to only the first
- `! x`: Negate a boolean
- `wrap x`: Wrap a value in a list
- `empty`: Produces the empty list
- `debug_enabled`: Can be overriden to enable debugging utilities
- `debug i x`: Display the value of `x` with the information tag `x`
- `assert i x`: Assert that `x` is true
- `assert_eq x y`: Assert that `x` and `y` are equivalent
- `is_atom x`: Determine whether a value is atomic (i.e: null, bool or a number)
- `is_str x`: Determine whether a value is a string
- `is_list x`: Determine whether `x` is a list
- `is_bool x`: Determine whether `x` is a bool
- `is_num x`: Determine whether `x` is a number
- `is_null x`: Determine whether `x` is null
- `len l`: Determine the length of a list
- `skip n l`: Skip the first `n` values in a list
- `nth n l`: Get the `n`th item in a list
- `in x l`: Determine whether `x` is in a list
- `split i l`: Split a list into two separate lists at the `i`th index

You can check `src/atto/core.at` for full documentation about what `core` provides.

## Tutorial

### Basic numeric operators:

```
fn main is
	+ 5 7
```

Yields: `12`

```
fn main is
	- * 3 3 5
```

Yields: `4`

### Printing values to the console:

```
fn main is
	print "Hello, world!"
```

```
fn main is
	print str 1337
```

### Receiving inputs from the user and converting them to a value:

```
fn main is
    print + "Product = " str
    	* litr input "second: "
          litr input "first: "
```

### Pairing values together into a two-component list:

```
fn main is
	pair 3 17
```

Yields `[3, 17]`

### Fusing lists together:

```
fn main is
	fuse pair 3 17 pair 5 8
```

Yields: `[3, 17, 5, 8]`

### Conditional expressions:

```
fn main is
	if true
		10
		5
```

Yields: `10`

```
fn main is
	if false
		10
		5
```

Yields: `5`

### Selecting the first value in a list:

```
fn main is
	head pair 3 17
```

Yields: `3`

### Selecting values trailing after the head of a list:

```
fn main is
	tail fuse 3 fuse 17 9
```

Yields: `[17, 9]`

### Converting a string into a value:

```
fn main is
	- 7 litr "3"
```

Yields: `4`

```
fn main is
	+ 7 litr "8.5"
```

Yields: `15.5`

```
fn main is
	= null litr "null"
```

Yields: `true`

### Defining a function with parameters:

```
fn add x y is
	+ x y

fn main is
	add 5 3
```

Yields: `8`

### Recursion to find the size of a list:

```
fn size l is
	if = null head
		0
		+ 1 size tail l
fn main is
	size fuse 1 fuse 2 3
```

Yields: `3`

## Optimisation

Currently, Atto's Rust interpreter performs virtually no optimisations. Despite that, I'll attempt to talk below about some ideas I've had that seem promising.

Atto's design does not permit the realiasing of values within a function, nor does it permit mutation. This, and the fact that the syntax is incredibly
quick to parse, makes it an extremely good potential target for a lot of optimisations. Inlining, constant propagation, CSE detection and tail call
optimisations are naturally easy to implement on top of Atto's design.

The lack of realiasing also means that Atto has an affine type system *by design*, without ever requiring compiler
support for move semantic analysis or anything like that.

The only real obstacles to some really impressive optimisation is its dynamic type system. However,
in a significant number of cases it's likely that types can be inferred at compile-time with specialized machine code emitted for each function depending on
the types passed to it.

I'm also working on ideas for a statically-typed version of Atto. However, I've yet to settle on a design that is sufficiently simple as to compliment the
current design.
