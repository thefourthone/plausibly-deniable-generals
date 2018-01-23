# Plausibly Deniable Generals

This project is the intersection of [Plausible Deniability](https://github.com/ad510/plausible-deniability) and [generals.io](https://generals.io). The basic idea is to create a way for custom clients to "cheat" in a fair and controlled way in a game very similar to generals.

The way it does this is by only having clients show the neccessary information for interacting with other players to the server and then upon finishing the game reveal the moves they played. Changing the moves they played midgame is totally acceptable, but there has to be a normal rule-following replay that matches the given midgame information.

## How do I play this?

Currently, you don't. There doesn't exist a server or client. If you stumbled onto this this and it seems interesting, I should have a basic server up pretty soon (within 4 weeks); if you want to help developing a simple webclient, you could open an Issue to chat we me about it. 

Long term, I don't expect to keep a server up (I might keep a testing server up for a year though), but I'll distribute server binaries which will make it easy to host your own server and play with friends.

## Why
Andrew Downing (the creator of Plausible Deniability) showed it to me a few years ago when and I couldn't figure out how to turn it into a game (and I still don't). 

From the readme of Plausible Deniability:
> But after I reached this point in summer 2014, I pretty much hit a brick wall because I have no clue how to turn this into an actual game, and neither does anyone I showed it to. That said, if I figure out a way to get unstuck (and simultaneously manage the necessarily complex codebase), I'd still love to turn this into a full multiplayer game. 

I really liked the idea of Plausible Deniability, but feel that the key idea wasn't fully explored within the RTS genre (because it is hard to make a good system for powerful "cheating"). I feel that a discrete semi-realtime strategy game can be more fully explored and may be able to shed light for the RTS side (which if well designed could be amazing).

So the goals of this project are to (roughly in order):
1. Make a fun game with a neat gimmick
2. Better understand plausible deniability (potentially getting ideas to help the original) 
3. Learn a bit about Rust and get some more general programming experience.
 
## Protocol

Because this is project really needs competeing clients to reach maximum potential, the protocol for interacting with the server is going to be fully documented, but I haven't finalized it yet.

### In progress protocol

All communication to the game server will be through websockets. All text messages will be broadcast to the current room with a prepended username + ": ". The first byte of every binary message describes the type of message; the rest of the message should be parsed according to that messages rules. 

The first byte is further separated into the first nibble being metadata and the second nibble being an arbitrary message type field.
So in binary the first byte would be separated into 5 areas `abcdeeee`,

|Label|Description|Values|
|-----|-----------|------|
|a|Direction of travel | 1 = server -> client, 0 client -> server|
|b|Which server needs to handle the message | 1 = meta server, 0 = game server|
|c|Info/Status message | 1 = Doesn't change internal state 0 = Changes internal state|
|d|Error| 1=there was an error, 0 = no error|
|e|Additional arbitrary data to separate different message types | See messages.json|

Before the more specific parsing of the different messages some sub-parts need to be covered.

|Name of sub-part | Description | How to parse | Example | 
|-----------------|-------------|--------------|---------|
|Bounded size (unsigned) integers| Integers that can't grow infinitely (or quite large)| 2's complement (or raw) integers with a specified number of bytes.|0xFFFF (2 byte unsigned) -> 65535|
|Unbounded unsigned integers|Numbers like the number of soldiers in a cell (can potentnially grow infinitely, but generally small)| Parse as an unsigned number and if it is the parse a bounded integer => a, then another of this type => b and the result is a+(b*(maximum bounded + 1))|0xFF0ABB (1 byte) -> 0xBB0A<br> 0xFF0AFFBBCC (1 byte) -> 0xCCBB0A<br> 0xF9 (1 byte) -> 0xF9|   
|String|ASCII text|(2 byte unsigned integer) as length then that number of bytes interpretted as ASCII text (perhaps going to UTF-8 at some point)| 0x000548656c6c6f -> "Hello"|
|Point (Or positive difference between two points)|Location on a board (with a already known width)|Parse unbounded int then use the remainder from dividing by the width to give the X coordinate and the the rounded down ratio be the Y coordinate.| 0xF0 (on a 7 wide board) -> (2,1)|

Notation: The parsing section following will use (u)int, unbound, string, and point to represent the respective types of data. the number at the end of type is how many bits the underlying (u)int is. Commas mean to parse the left, then the right.

| Name of type | Description | When used | How to parse |
|--------------|-------------|-----------|--------------|
| updateCells | Used to share knowledge of the current board state. | Each turn each player must send all of the cells they own that might be seen by another player; the server sends neccessary updates to the client to show other players movements | uint8 player, point8 firstLocation, unbound8 value, (point8 diffFromLast, unbound8 val) for each cell| 
| move | Make a move into unoccupied or enemy territory | Client -> Server only, when another player makes a move that will be seen in the updateCells message. | point8 location, uint8 direction, unbound8 units |
| status | Used to update clocks, land owned, etc | Server -> Client only, once every turn | (uint16 time, unbound troops) for each player|
| replay | A list of moves | Used at the end to validate no cheating | uint8 player, (unbound8 numberOfSkips, point8 location, uint8 direction, unbound8 units) for each period of activity |
