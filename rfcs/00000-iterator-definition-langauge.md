- Feature Name: `iterator_definition_language`
- Start Date: 2020-04-18
- RFC PR: [remits/rfcs#0000](https://github.com/badtuple/remits/pull/0000)

# Summary
[summary]: #summary

This RFC introduces a simple "pipe and filter" style language for defining Iterators. This allows users to manipulate streams off of Logs at the _Stream_ level and not the Message level that current iterators require.

Users will still be able to define arbitrary transformation of Messages via Lua functions, but those functions will be registered as Filters that can be included in an Iterator.

# Motivation
[motivation]: #motivation

All transformations are technically expressible via composition of Map, Filter, and Reduce iterators. Just because it's possible doesn't mean it's ergonomic however. Common scenarios are so complex to implement at the Message level that Remits would seem incredibly limited to the prospective user.

Consider a Log of events containing an integer field `val`. The user wants to get the average `val` for each hour. Implementation at the Message level, the user would use a Reduce iterator to collect the `val` fields for the current hour and 1 from the next, a Filter iterator ontop of that determines if there's a single `val` for the next hour (meaning the previous hour had been fully collected) and let only that Message through. Ontop of _that_ they'd need a Map to then take the Messages passed from the Filter and average them for the previous group and return it. Not the most ergonomic at all, and uses quite a bit of memory unnecessarily.

At the _stream_ level, the user creates this iterator ontop of their Log:

```
LOG_NAME | bin(1h) | avg(msg.val)
```

This is more approachable and flexible. It also allows us to optimize data access and memory usage forknown filters. This also simplifies implementation of custom Lua functions, because then the function only takes a Message and returns a Message. There's no longer any need to have different formats for Map, Reduce, and Filter iterators.

# Design
[explanation]: #explanation

The language is purposefully kept simple. There is a single input at the beginning which refers to either a Log or other Iterator. One or more Filters can be added after, separated by a pipe `|`.  A `Filter` is a function that takes 1 or more Messages and returns 0 or 1 Message. There will be builtin Filters, but a user can define a custom Filter in Lua if they desire. User defined Filters are prefixed with an `@` to distinguish them from builtins.

The language will also need to support boolean expressions, primitive literal instantiation, and likely basic math. This is because Filters often need conditions.  Consider this query:

```
LOG_NAME | filter( msg.val >= 4 )
```

This query passes the stream into the `filter` Filter. From a language standpoint, it also includes the instantiation of an expression, a primitive literal `4`, a value access `.val`, and a binary operator `>=`.  Because these are necessary things to be able to define, the language has to be slightly more fleshed out than just splitting on `|` and mapping the filter names to an in-memory representation.

This design removes the existing idea of Map, Filter, and Reduce Iterator types. An Iterator is now just an input and a list of Filters. This hopefully simplifies a user's idea of an Iterator.

Iterators would still be queried the same way they are now. This only addresses the creation of Iterators, and the level of abstraction they operate on.

# Drawbacks
[drawbacks]: #drawbacks

This requires users to learn a new query language. Ideally it's kept simple, but it's still something else the user has to learn. Ways to mitigate this would be to use filters that line up with established libraries such as Lodash, jq, Rust's Iterator package, or Ruby's Enumerable class.

# Alternatives and Prior Art
[rationale-and-alternatives]: #rationale-and-alternatives

The most obvious alternative is to keep down the route of keeping Map, Reduce, and Filter Iterators. Technically everything is expressible and the implementation is conceptually simple. I believe that this leads to an undue burden on the user as explained above.

A second alternative would be to implement or use a premade lisp instead of a pipe-filter langauge. Plenty of mature lisps exist for Rust already. One upside is that Lisp is very expressive when it comes to expressing chains of functions, and so making a 1:1 translation of Lodash or similar would be trivial. Another upside is that since the lisp would be a full language, we'd be able to use that to code Filters instead of Lua. I prefer Lua on it's own, but simplifying back down to one language might be worth it. However Lisp just fundamentally turns me off...it's hard to read with it's lack of syntax and I think people not being able to eye-ball an Iterator "query" and instantly know what's happening would deter users.

One common path taken by similar systems is to create a SQL-like language optimized for streaming. Examples are Kafka's KSQL and Materialize.io's adaptation of ANSI SQL for a constantly materializing view. There's the upside that people know and understand SQL and SQL-likes. However I believe that much of the simplicity of Remits comes from the fact that it's not trying to abstract away the idea that it's a stream/timeseries. By being direct with the pipeline format it's easier for users to reason about.

jq is a very interesting case. It allows manipulation of JSON accessed via a stream. Some commands require consuming the full stream, which would not be allowed in Remits, but due to CBOR's similarity to JSON, it's conceivable that the majority of jq's language could be ported and used _as_ the Remits processing language. This is alot more work however. I believe jq is turing complete and can be used for more than queries. It also allows arbitrary nesting of logic. Having a simpler, less-powerful language is a plus for us. It allows us to optimize for the common use case and allow custom Lua Filters as an escape hatch for more flexible queries.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

* Is there an open source and adopted language that is simple and optimized for our use case? While implementation wouldn't be hard for a simple pipe-and-filter langauge, it'd be great if we could piggyback and reuse an existing language and it's documentation.

* What filters do we ship with initially? I believe a few very simple ones should be enough too start with.

* Exact syntax is still to be determined. Especially around primitive type literals and how value access within the message works.
