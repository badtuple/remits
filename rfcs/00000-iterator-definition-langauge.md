- Feature Name: `iterator_definition_language`
- Start Date: 2020-04-18
- RFC PR: [remits/rfcs#0000](https://github.com/badtuple/remits/pull/0000)

# Summary
[summary]: #summary

This RFC introduces a simple "pipe and filter" style language for defining
Iterators. This allows users to manipulate streams of Messages at the _Stream_
level, instead of Message level.

Users will still be able to define arbitrary transformation of Messages via Lua
functions, but those functions will be registered as Filters that can be included
in an Iterator.

# Motivation
[motivation]: #motivation

All transformations are expressible via composition of Map, Filter, and Reduce
Iterators. Those expressions often turn out to be incredibly unergonomic.
Common queries can be so complex to implement from the Message level that Remits
will seem incredibly limited to prospective users.

For example, a User has a Log of Messages containing an integer field `val`.
They want to get the average `val` for each hour.
Implementation at the Message level requires multiple iterators:

* a Reduce Iterator to collect the `val` fields grouped by hour
* a Filter Iterator that will filter out any aggregates that do not have exactly one `val` in the latest hour. That means the previous hour had been fully collected.
* a Map Iterator that takes the aggregate and averages the `val`s for the second to latest hour.

This is not trivial at all, and uses quite a bit of memory unnecessarily.

At the _stream_ level, the user creates this iterator ontop of their Log:

```
LOG_NAME | bin(1h) | avg(msg.val)
```

This is more approachable and flexible. It also allows us to optimize data
access and memory usage for known filters.

This also simplifies implementation of custom Lua functions. Filters only take
Messages and only return Messages. The different formats for current Map,
Reduce, and Filter iterators are eliminated.

# Design
[explanation]: #explanation

The language is purposefully simple. A Log or Iterator identifier begins the query.
Zero or more Filters can be appened, separated by a pipe `|`. A Filter is a
function that takes 1 or more Messages and returns 0 or more Messages. There are
builtin Filters, and users can define a custom Filters in Lua. User defined
Filters are prefixed with an `@` to distinguish them from builtins.

The language also needs to support boolean expressions, primitive literals, and
basic operators. This is to allow passing conditions to Filters.

Consider this query:

```
MY_LOG | filter( msg.val >= 4 )
```

Messages from `MY_LOG` are passed to the `filter` Filter. It also includes the
instantiation of an expression, a primitive literal `4`, a value access
`msg.val`, and a binary operator `>=`. This means the language has to be
slightly more fleshed out than just splitting on `|` and mapping the filter 
ames to an in-memory representation.

The existing Map, Filter, and Reduce Iterator types are removed. Iterators are
now just lists of Filters. This should simplify understanding of Iterators.

Iterators are still queried as they are now. This only addresses Iterator
creation, and the level of abstraction they operate on.

# Drawbacks
[drawbacks]: #drawbacks

This requires users to learn a new query language. Ideally it's kept simple, but
it's still something the user has to learn. Using familiar filters from known
libraries like Lodash, jq, Rust's iterator package, or Ruby's Enumberable class
might help ease learning.

# Alternatives and Prior Art
[rationale-and-alternatives]: #rationale-and-alternatives

The most obvious alternative is to keep Message level Iterators only. Everything
is still expressible and conceptually simple. However this places undue burden
on the user as explained above.

Another alternative is to implement or use a premade lisp instead of a
pipe-filter langauge. Many mature lisps exist for Rust already. Lisps are very
simple and express function chains well. Making a near 1:1 translation of Lodash
or similar would be simple. Since a lisp would be a full language, it could be
used to define Filters instead of Lua. I prefer Lua on it's own, but unifying
back to one language might be worth it. However Lisp just fundamentally turns me
off. It's lack of syntax makes it hard to read. If users cannot eye-ball an
Iterator and intuitively understand it, they will be deterred.

One common path taken by similar systems is to create a SQL-like language
optimized for streaming. Examples are Kafka's KSQL and Materialize.io's
adaptation of ANSI SQL for constantly materializing views. People's familiarity
with SQL and SQL-likes is a plus. However much of the simplicity of Remits comes
from not trying to abstract away the idea that it's a stream/timeseries. A
transparent pipeline format is easier for users to reason about.

jq is a language that manipulates JSON streams and may fit this use case.
Commands that consume the full stream cannot be supported, but CBOR's similarity
to JSON means a subset of jq could be used as Remits' processing language.
I do think that jq's syntax is harder to understand than a pipeline language.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

* Is there an open source/adopted language that is simple and optimized for our use case? It'd be great to reuse an existing language and it's documentation.
* What filters do we ship with initially? A few very simple ones should be enough to start.
* Exact syntax needs to be determined. Especially around primitive type literals and value accesses.
