# winRing0-rs

This is an attempt to create a wrapper around the winRing0 driver. It is mostly based upon [OpenHardwareMonitor](https://github.com/openhardwaremonitor/openhardwaremonitor).

## Packages

### `win-kernel-driver`

[`win-kernel-driver`](https://alex-dow.github.io/winRing0-rs/doc/win_kernel_driver/index.html) is a utility crate to install and communicate with windows device drivers. It is based on the KernelDriver class from OpenhardwareMonitor. For more information see the [KernelDriver class](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/Hardware/KernelDriver.cs).

### `win_ring0`

[`win_ring0`](https://alex-dow.github.io/winRing0-rs/doc/win_ring0/index.html) is a wrapper around the winRing0 driver itself. It is under development and does not support every feature for now.

## Usage

The crates are not currently published on crates.io and are under development. If you would like to use them anyways you can link to them manually:

```
[target.'cfg(windows)'.dependencies]
win-kernel-driver = { git = "https://github.com/alex-dow/winRing0-rs" }
win_ring0 = { git = "https://github.com/alex-dow/winRing0-rs" }
```

You could potentially leverage these crates with the [x86 crate](https://docs.rs/x86/0.33.0/x86/).
