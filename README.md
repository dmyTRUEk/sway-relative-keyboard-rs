# README

## About
`sway-relative-keyboard-rs` is program for [Sway](https://swaywm.org/),
that remembers your keyboard layout for each window and restores it on focus change.



## Installation
### Build from source:
1. Clone source code from this repository:  
   ```
   git clone https://github.com/dmyTRUEk/sway-relative-keyboard-rs
   ```

2. Compile it:  
   1. Install `rust` (and `cargo`) by your preffered method.

   2. Compile:  
      ```
      cargo build --release
      ```

   After successful build, you can find binary here:  
   ```
   path_to_src/target/release/sway-relative-keyboard-rs
   ```

3. (**Optional**) Move binary to preffered folder.

4. Add binary to sway "startup":  
   In your sway config file add this line:  
   ```
   exec path/to/sway-relative-keyboard-rs
   ```  
   and then restart your pc (or sway?).

   For example:  
   ```
   exec $HOME/.local/bin/sway-relative-keyboard-rs
   ```

   If you want to get it work without restart,
   use `exec_always`, then reload your config file,
   then change `exec_always` back to `exec`,
   so it executes only on sway startup.

