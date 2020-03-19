Design
======

This document is a high level description of Remits' protocol/api, operations,
and some intended features. It's to help communicate intent and direction during
early development.

Nothing is implemented yet, so it's basically all up in the air.

## Description of Remits

Remits is an "Iterator Server". At it's heart it has 3 primary constructs:

  1. A *Message* is a list of bytes stored in a Log.
  2. A *Log* is a persisted, append-only list of Messages.
  3. An *Iterator* applies a Map/Filter/Reduce operation over a Log or Iterator
     and returns the modified Messages.

A client will be able to push Messages to a Log, and then create an Iterator
over it to query them back out.

Each Message has an Offset which represents it's index in the Log. When using
an Iterator, you can choose to iterate from the beginning, the end, or a certain
Message's Offset. This allows you to store your place in an Iterator and then
resume from that spot at a later time.

## Messages

Messages should be very simple.
We don't want to enforce a serialization format on the user.

This means Remits can be used as a timestream/metrics database where each
Message is a 64 bit integer, as a message queue where each Message is JSON, or
a custom audit log where Messages are arbitray binary objects.

## Log

Logs can only be:
  * Created
  * Deleted
  * Pushed to

You cannot delete or update a Message from a log. You can only push a new one.
You cannot query a log directly. You must always use an iterator.

## Iterators

Iterators query Messages from Logs.

You define an Iterator via a Lua function (choice of Lua is up for discussion.)
There are three types of Iterators:
  1. A *Map* Iterator transforms each Message before sending it to the client. 
  2. A *Filter* Iterator takes each Message and returns a boolean. If `false`
     is returned, the Message is not sent to the client. If `true` is returned,
     the Message is sent to the client.
  3. A *Reduce* Iterator takes two arguments: the return value from the last
     iteration, and the current Message. This is useful for aggregating Messages
     into sums, lists, or other aggregates.

By default, iteration happens over the log at the time it is queried.
However you can optionally make an Iterator "Indexed".  An Indexed Iterator
will persist each result of the Iterator to disk. This allows faster iteration
and less work if you need to query the same Iterator again.

This also allows you to create a base for another Iterator to use.
For example, say you had a Log of payment attempts. You could create an Indexed
Filter Iterator to persist only attempts that failed. You could then create
other Iterators ontop of that so that you only have to iterate through the
filtered set and not the entire Log while doing analysis.

## Protocol

Clients communicate with Remits over a TCP connection.
Once a connection is opened, the client can send frames of data over the wire.

The first 32 bits of a frame is an unsigned 32 bit integer denoting the length
of the data that follows it in bytes:

+- len: u32 -+----- data -----+
| 0x0000000d |  LOG ADD mylog |
+------------+----------------+

Since the data we are sending (`LOG ADD mylog`) is 13 bytes long, we send over
the binary encoding of 13 (`0x0000000d`.)

Each Frame can contain only 1 command.

## Commands

Ideally, Remits will have few commands. We should try to limit them as much as
possible. Here's a list of commands I can think of right now, but this is very
much a work in progress and flexible as the project proceeds:

Command                                             | Description
----------------------------------------------------+----------------
`LOG ADD <log name>`                                | Create a new Log
----------------------------------------------------+----------------
`LOG DEL <log name>`                                | Delete a Log
----------------------------------------------------+----------------
`MSG ADD <log name> <message binary>`               | Add a Message to a log
----------------------------------------------------+----------------
`ITR ADD <log name> <iterator name> <lua function>` | Create an Iterator
----------------------------------------------------+----------------
`IDX ADD <log name> <iterator name> <lua function>` | Create an Indexed Iterator
----------------------------------------------------+----------------
`NXT <iterator name> <0|-1|message offset>`         | Get the next Message after a specific Offset.
                                                    | `0` will always refer to the first message,
						    | `-1` will refer to the latest message
----------------------------------------------------+----------------
`PRV <iterator name> <0|-1|message offset>`         | Get the Message before a specific Offset.
						    | `-1` will refer to the latest message
