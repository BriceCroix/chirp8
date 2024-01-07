# Chirp8

Chip-8 emulator library written in Rust.

The library is compatible with `no_std` environments and can be used to create handheld consoles on microcontrollers !

## Examples

- `minimal.rs` : Provides the bare minimum to use this library. Very suitable for embedded projects.
- `with_piston.rs` : An example of desktop application using the `piston` library to run your favorite chip-8 games on your computer.

## Crate features

| **Feature name** | **Description**                                                                                       | **Default-enabled** |
| :--------------: | :---------------------------------------------------------------------------------------------------- | :-----------------: |
|     `alloc`      | Allocates the objects that use the most memory on the heap (`Vec<T>`) instead of the stack (`[T; N]`) |         yes         |
|     `xochip`     | Adds support for XO-chip. The biggest difference is the RAM size that grows from 4kb to 64kb.         |         yes         |

## Testing

This library uses [Timendus' tests suite](https://github.com/Timendus/chip8-test-suite.git) as a git submodule.
This is only needed if you wish to contribute to this library.
Please run the following to pull the submodules :

```sh
git submodule update --init --recursive
```