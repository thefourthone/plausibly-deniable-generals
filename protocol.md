First byte:
abcdeeee

a: direction, 1 = server -> client, 0 client -> server
b: if it is for the meta server (aliasing, join room/lobby)
c: info/status? 1 = Doesn't change internal state 0 = Changes internal state
d: error? 
e: message type
  gameplay: 
    normal:
      update cells             - a000 0001
      move made                - a000 0010 
      status (time, points, etc) 1010 0011
      replay                     a000 0100
      leave game
    error:
      missing data             - a?011 1000  
      banned (from game)       - 1011 1001
      internal error           - 1011 1101 
  message server:
    username
    ranking
    join/create room
    join lobby
    start game
    leave room
    ping
    info
      error 
        malformed message
        bad userkey
        user banned
        room error
        bad connection / reconnect
        
      
