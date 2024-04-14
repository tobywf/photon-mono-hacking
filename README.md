# Dump the Anycubic Photon Mono 4k firmware

This repository contains my notes and code for dumping the firmware of a Anycubic Photon Mono 4k 3d printer (at least the MCU and Flash, not the FPGA). It's heavily based on nviennot's [reverse engineering the Anycubic Photon Mono 4K](https://github.com/nviennot/reversing-mono4k) guide.

The only innovation I have is that this repo contains code to dump the external flash via RTT instead of the semi-hosting method in nviennot's guide. The benefits are:

* This bypasses the issue that neither [the `cortex-m-semihostring` crate](https://github.com/rust-embedded/cortex-m) nor [`probe-rs`](https://github.com/probe-rs/probe-rs/blob/v0.23.0/probe-rs/src/rtt/syscall.rs#L4) currently implements some or all of the `SYS_OPEN`/`SYS_WRITE`/`SYS_CLOSE` syscalls.
* This does not involve setting up semi-hosting with OpenOCD, which I found to be a pain.

Fortunately, dumping the flash via RTT is simple and quick.

## Hardware requirements

* The 3d printer itself.
* A debug probe supported by `probe-rs`.
* Wires to connect the debug probe to the 3d printer.

## Software requirements

* [Rust](https://rustup.rs/).
* [`probe-rs`](https://probe.rs/). This repo was tested with `probe-rs 0.23.0`, and as of 2024-04-13 there are breaking changes in the repository around the format of `Embed.toml`.
* The software for the debug probe, e.g. [OpenOCD](https://openocd.org/pages/getting-openocd.html) for ST-Link or [J-Link](https://www.segger.com/products/debug-probes/j-link/).

## How to dump MCU firmware

### OpenOCD

Dumping the MCU firmware via OpenOCD isn't too difficult.

First, find the OpenOCD script for the interface (debug probe). For example, on macOS/with Homebrew for an ST-Link interface, this is in `/opt/homebrew/share/openocd/scripts/interface/stlink.cfg`.

Second, find a suitable OpenOCD script for the target (MCU). For example, on macOS/with Homebrew, and almost compatible target is `/opt/homebrew/share/openocd/scripts/target/stm32f1x.cfg`. The only difference is that the GigaDevice GD32F307VET6 CPU TAP ID is slightly different than STM32F1X chips:

```diff
<       set _CPUTAPID 0x2ba01477
---
>       set _CPUTAPID 0x1ba01477
```

I have included a patched `stm32f1x.cfg` in the `openocd` directory, but it's probably best to patch whatever version on the script comes with your OpenOCD installation. Alternatively, OpenOCD scripts for the GD32F3x family are out there.

Start OpenOCD:
```bash
openocd \
    -f '/opt/homebrew/share/openocd/scripts/interface/stlink.cfg' \
    -f './openocd/stm32f1x.cfg'
```

And issue the `dump_image` command (N.B.: 0x80000 == 512 KiB):

```bash
echo 'dump_image ./firmware/mcu.bin 0 0x80000' | nc localhost 4444
```

It's also possible to verify the firmware against an image dumped by someone else:

```bash
echo 'verify_image ./firmware/mcu.bin 0 bin' | nc localhost 4444
```

For more information, see the [OpenOCD documentation on image loading commands](https://openocd.org/doc-release/html/General-Commands.html#Image-loading-commands). I have found it more reliable to set the `address` parameter to `0` instead of the actual address of the MCU's flash of `0x08000000`.

### probe-rs

Currently, `probe-rs` cannot dump firmware. (The `download` command is for downloading from the host to the target.)

## How to dump the external flash

Assuming the interface/debug probe is supported by `probe-rs`, simply run `cargo embed`. This will build the code, upload the code, and start an RTT session/terminal. The RTT information is defined in `src/main.rs` and `Embed.toml`. The RTT terminal should print the dumping progress in the first tab; the second tab is used to send the flash contents to the host. The logs of the dumping operation are also configured to be saved (via `Embed.toml`), but note this only happens once the RTT terminal is closed! The logs will appear in the `logs/` directory, one for the text and one for the binary.

The dumping operation itself takes some time depending on the speed of the probe (I think). For me, it was around 6 minutes. This roughly matches the semi-hosting approach.

To compare the dumped flash against an image dumped by someone else, run:

```bash
cmp 'firmware/ext.bin' 'logs/photon-mono-dump_STM32F103ZE_1713048730062_channel1.dat'
```

## How to restore the MCU firmware

### OpenOCD

Start OpenOCD as described in the dumping procedure. Then, run the `load_image` command:

```bash
echo 'load_image ./firmware/mcu.bin 0 bin' | nc localhost 4444
```

It's also a good idea to verify the image was written correctly, again see the dumping procedure.

### probe-rs

This should work:

```bash
probe-rs download --chip 'STM32F103ZE' --base-address '0x08000000' --verify --format 'bin' './firmware/mcu.bin'
```

It always produced an error for me, but after verifying the image with OpenOCD, it seems to have worked. So IDK, maybe just use OpenOCD.

## How to restore the external flash

I don't know an easy way to do this. Presumably use the same method that was used to modify the external flash in the same place.

The main issue is that down channels seem to be poorly supported in `probe-rs`. Otherwise, it should be possible to use a similar approach to dumping the firmware, but in reverse by piping data via RTT from the host to the target. This is necessary, as the external flash size is much larger than the MCU flash size.

Alternatively, With access to the MCU it should be possible to write code to load the external flash image from a USB drive, and restore the flash that way.
