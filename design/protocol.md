Client/Server Protocol
======================

Clients communicate with Remits over a TCP connection. Once a connection is
opened, the client sends a request frame to Remits, and receives a response
frame in return.

## Frames

Frames sent over the TCP connection conform to the following format:

```
+- len: u32 -+- kind: u8 -+- code: u8 -+--- payload ---+
| 0x00000000 |    0x00    |    0x00    |  cbor binary  |
+------------+------------+------------+---------------+
```

### Len

The first 4 bytes of a frame represent the length of the rest of the frame.
It is encoded as an unsigned 32bit integer. The Len field includes the length
of payload as well as the size of the Kind and Code fields.

### Kinds

A Frame's *Kind* represents the overall purpose of the Frame, and is represented
by the first byte in a Frame. Current kinds are:

| Kind          | Byte | Description                                                             |
|---------------|------|-------------------------------------------------------------------------|
| Request       | 0x00 | A request made from the client to a Remits server                       |
| Info Response | 0x01 | A response made from the Remits server to a client containing a message |
| Data Response | 0x02 | A response made from the Remits server to a client containing data      |
| Error Response| 0x03 | A response made from the Remits server to a client containing an error  |

### Codes

A Frames *Code* is used to determine the meaning of the rest of the Frame.
A Code for a Request is used to determine the type of query being made.
A Code for an Error Response represents the type of error.
A Code for an Info Response and Data Response is always 0x00.

The Frame Code is the second byte in the request.

### Payload

The rest of the frame is data encoded in CBOR format. The schema of that data
depends on the Frame's Kind and Code.

## Request

A Request has the kind `0x00`. Different queries and operations have different
codes. The format for the data is dependant on the operation.

### Log Show

The Log Show operation shows metadata associated with a Log.
The payload is encoded in CBOR and follows this format.

```
{
  "log_name": String
}
```

### Log Add

The Log Add operation creates a new Log.

```
{
  "log_name": String
}
```

### Log Delete

The Log Delete opration deletes a Log.

```
{
  "log_name": String
}
```

### Log List

The Log List operation lists all existing Logs.
It has an empty payload, 0 bytes long.

### Message Add

The Message Add operation adds one or more Messages to a Log.

```
{
  "log_name": String,
  "messages": Array of CBOR encoded messages to add to the Log
}
```

Each message in the "messages" array should be individually CBOR encoded,
_before_ the full payload is encoded in CBOR.

### Iterator Add

The Iterator Add operation adds an iterator to a Log

```
{
  "log_name": String,
  "iterator_name": String,
  "iterator_type": String,
  "iterator_func": String,
  "indexed": Boolean,
}
```

### Iterator List

The Iterator List operation lists all Iterators.
An optional Log name can be included to scope the listed Iterators to those
attached to a particular Log.

```
{
  "log_name": Optional<String>
}
```

### Iterator Next

The Iterator Next operation gets up to `count` messages from an Iterator,
starting at a specific Message ID.

```
{
  "iterator_name": String,
  "message_id": Integer,
  "count": Integer
}
```

Message ID `0` will always return the first message in the Iterator.
Message ID `-1` will always return the last message in the Iterator.
