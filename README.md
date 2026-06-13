# Installation

This installation is really not beginner-friendly yet. I'm sorry!

1. You will need `ffmpeg` installed. If you are unsure whether it is installed, type `ffmpeg -version` into your terminal. If it's not installed, look up how to install it for your OS.
2. Build the project for your computer's architecture using `cargo`. Type `cargo build --release` into the terminal while at the project directory (1 directory above /src). Then, find the application in /target/release/ (Windows has vidyffy.exe, others have vidyffy)
3. Run this application. It can be placed anywhere on your computer.


# Known issues:
- Gif system is not fully implemented. Can't actually save gifs
- Gif player rendering is bizarre and not in the right location on the screen.
- User can interact while working on an operation, which might break things.
## Code issues
- Interchangably using usize and u32 often. Don't do this!
