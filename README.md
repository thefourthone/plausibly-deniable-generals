# Plausibly Deniable Generals

This project is the intersection of [Plausible Deniability](https://github.com/ad510/plausible-deniability) and [generals.io](https://generals.io). The basic idea is to create a way for custom clients to "cheat" in a fair and controlled way in a game very similar to generals.

The way it does this is by only having clients show the neccessary information for interacting with other players to tghe server and then upon finishing the game reveal the moves they played. Changing the moves they played midgame is totally acceptable, but there has to be a normal rule-following replay that matches the given midgame information.

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
|e|Additional arbitrary data to separate different message types | 0xA |




