# RustyLife
An simple implementation of Game of Life in Rust.

This is uses several ideas from the implementation of snake from Redox-OS (https://github.com/redox-os/games/tree/master/src/snake).

At the current state the game works, but could use several improvements.

## Features.

We use a "toriodal" world in the sense that the left boundary is "glued" to the right boundary and the bottom boundary is "glued" to the top boundary. A glider leaving hitting the bottom boundary will reappear at the top boundary. See below screenshot for example.

## Screenshot
<p align=center> 
  <img src = https://media.giphy.com/media/dB1Go9uCBS72Eck09h/giphy.gif alt="Rusty Life Glider Gun">
</p>



## TO-DO

- Switch between toriodal mode, and "dead-outside-of-screen" mode.
- Allow pause and update during run.
- Get accepted into Redox-OS game collection.



