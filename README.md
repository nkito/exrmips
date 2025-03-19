# MIPS Emulator (Rust version)

## Overview

This is a Rust version of MIPS32R2 emulator [exmips](https://github.com/nkito/exmips "exmips").
This emulator works as a console application and also works in a Web browser as WebAssembly codes.

OpenWrt, a well-known Linux distribution for home routers, works in the emulator of both console application version and Web browser version.

## Build

### Console application
A console appliation is obtained as follows.
```
$ cargo build --release
```
Note that a program binary obtained with "debug" build may not work properly. It is a known bug.
The simulator carries out additions with overflow intensionaly. However, such overflow causes "panic" when a debug-build is used.

### WebAssembly package
WebAssembly version of the emulator for used in Web browsers is obtained as follows.
```
$ wasm-pack build --target web
```
``wasm-pack`` is necessary to compile it. It can be installed by ``cargo install wasm-pack``. 

### ROM image

The emulator works with a ROM image. The generation of a necessary ROM image is described in [exmips](https://github.com/nkito/exmips "exmips").

## Usage

The emulator works using a SPI flash ROM image.

```
$ cargo run --release u-boot/firm_u-boot.bin
```
Twice inputs of Ctrl+C halt the emulator.


