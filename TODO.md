0) [x]  Setup integration tests.
1) [x]  Change logs to store messagepack directly and validate it on receipt.
2) [ ]  Create Iters that don't run the Lua functions, but allow you to get messages out (this is easy, I've done it like 3 times in different incarnations)
3) [ ]  Write the MessagePack -> Lua type conversions. This is gonna be the hard part.
4) [ ]  Pass those types into Lua and execute the scripts.  At this point, Iterators are effectively done.

