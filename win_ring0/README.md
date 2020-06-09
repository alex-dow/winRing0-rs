# win_ring0

This crate provides a wrapper around the winRing0 windows kernel driver.

This driver is not complete. Currently only the `readMsr()` function is supported.

## Misc Information

Bundled with the crate are kernel drivers taken from [OpenHardwareMonitor](https://github.com/openhardwaremonitor/openhardwaremonitor), which originally seems to hail from [OpenLibSys](https://openlibsys.org/manual/).

## Method of Operation

This library will install a windows service called "winRing0_1_2_0" but it will not automatically uninstall it. You will need to manage the driver's lifecycle yourself.

## Usage

See example project. Needs to be run as administrator.
