# Rofi-randr
A (massively overengineered) [rofi script](https://man.archlinux.org/man/rofi-script.5.en)
to manage randr (Resize And Rotate) features of some display servers. 

## Backends
This program supports multiple backends. It automatically determines which one to use, 
but you can override this behaviour by setting `DPY_SERVER_OVERRIDE` in your environment.

* `libxrandr` - Uses the [xrandr crate](https://crates.io/crates/xrandr) 
to call libxrandr bindings.
* `swayipc` - Uses the [swayipc](https://crates.io/crates/swayipc) crate
to issue commands to sway.
* `xrandr_cli` - Just calls xrandr in a subprocess.

## Usage
Compile using `cargo build --release`. Then call rofi using:
```
rofi -modi "randr:/path/to/executable" -show randr
```

**NOTE:** When using wayland backends (like `swayipc`), it is best to use the 
[wayland fork](https://github.com/lbonn/rofi#wayland-support) of rofi.

## Features
The following features are supported:
* Enable outputs
* Disable outputs
* Set primary output
* Change resolution
* Change refresh rate
* Position outputs
* Rotate outputs

Backends can specify which of these features they support. Sway, for example,
has no 'primary display'.


## TODO
* Make a general wayland backend, perhaps using `wayland-client`.
* In the meantime, maybe make a `wlr-randr_cli` backend.
* Redo error structure.
