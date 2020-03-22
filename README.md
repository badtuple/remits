# Remits  &nbsp;![Rust](https://github.com/badtuple/remits/workflows/Rust/badge.svg) [![Discord](https://img.shields.io/discord/691125113659195474)](https://discord.gg/gSVR24K)

Remote Iterator Server

Remits is a timeseries database / log abstraction that allows you to query via intuitive map, reduce, and filter functions called _Iterators_.

Remits is built to be fast, lightweight, and reliable. It is intended to be easy to setup, manage, and reason about.

*Remits is still very young. Feel free to play around with it, contribute, and help define it's future...but don't use it in production yet.*

## Installation

To build the server:

```sh
git clone https://github.com/badtuple/remits
cd remits
cargo build --release --bin remits
```

To build the CLI Client for testing/administration:

```sh
cargo build --release --bin client
```

## Design

Overall design documentation is hosted in the `design` folder in the root of this repo.

From a high level though, Remits only has 3 primary constructs:

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

## Contributing

Want to contribute to Remits?
Feel free to jump into [our chat](https://discord.gg/gSVR24K) or open a github issue. We'll be happy to help find a good place to get started!
Remits is still young, so there's alot of low hanging fruit and definitional work to do.

## Questions

For now, if you have any questions please open up a github issue.
