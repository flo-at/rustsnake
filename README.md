# RustSnake
This is a simple terminal-based Snake game implemented in Rust without dependencies.\
To start the game type `cargo run`. Control the snake using W/S/A/D.

![Screenshot text](/media/screenshot.jpg?raw=true)

The reason I didn't use external crates is that I wanted to learn about the different corners of the language without them being hidden behind some nice and easy interfaces. This comes with some downsides though. The code is not very portable/cross-platform and won't run under Windows without some changes.

Even though it's a simple game, it does cover many of Rust's core features like multi-threading using channels, FFI to reconfigure the terminal, unit testing, and many more.
