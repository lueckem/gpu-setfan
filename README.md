# gpu-setfan

[![CI](https://github.com/lueckem/gpu-setfan/actions/workflows/ci.yml/badge.svg)](https://github.com/lueckem/gpu-setfan/actions/workflows/ci.yml)
[![Version info](https://img.shields.io/crates/v/gpu-setfan.svg)](https://crates.io/crates/gpu-setfan)

Controls GPU fan speed to maintain a target temperature.
Instead of providing a fan curve, like many other tools require, you only specify the temperature at which the GPU should run under load.
`gpu-setfan` automatically finds the smallest fan speed to maintain that target temperature, which minimizes fan noise.

Supports multiple GPUs: controls the fan speed of each detected GPU separately.

**Requirements**:
- Windows or Linux (x86_64)
- At the moment only **Nvidia** GPUs are supported


## How it works
Internally, a [PI-controller](https://en.wikipedia.org/wiki/Proportional%E2%80%93integral%E2%80%93derivative_controller)
is used to find the control (i.e., the fan speed) that steers the system to the so-called *setpoint* (i.e., the target temperature).

The general behavior is as follows:
- At the start, the fans are off (speed = 0%).
- The fans get turned on when the temperature exceeds the `--fan-on` temperature.
- Then the fan speed is controlled with a PI controller to maintain the target temperature. The fan speed is always larger than `--min-speed` to ensure stable operation.
- When the temperature falls below the `--fan-off` temperature, the fans get turned off again.

Because the `--fan-off` temperature is smaller than the `--fan-on` temperature, frequent on/off cycling is prevented (hysteresis).


## Installation
Download a precompiled binary from the [release page](https://github.com/lueckem/gpu-setfan/releases).

If you have Rust installed, you can also get the latest release from [crates.io](https://crates.io/crates/gpu-setfan) using the cargo package manager:

```sh
cargo install gpu-setfan
```

## Usage
`gpu-setfan` is a command line tool, so you interact with it via the terminal:

```sh
gpu-setfan [OPTIONS] [TARGET_TEMPERATURE]
```
Note that you likely need elevated privileges to set GPU fan speed, i.e., open the terminal as administrator on Windows, or run with `sudo` on Linux.

### Arguments

| Argument | Description | Default |
|---|---|---|
| `TARGET_TEMPERATURE` | Target temperature in °C | `80` |
| `--fan-on` | Temperature at which fans turn on; must be < target | target − 10°C |
| `--fan-off` | Temperature at which fans turn off; must be < fan-on | fan-on − 5°C |
| `--min-speed` | Minimum fan speed in % once fans are on (0–100) | `30` |

### Examples
Keep GPU at 75°C, turning fans on at 65°C and off at 60°C:

```sh
gpu-setfan 75
```

Keep GPU at 75°C, turning fans on at 68°C and off at 62°C:

```sh
gpu-setfan --fan-on=68 --fan-off=62 75
```


## Running on startup

### Linux
The easiest way to run `gpu-setfan` automatically on startup is to use a systemd service.
1. Download the `gpu-setfan` binary and move it to `/usr/local/bin/gpu-setfan`.
2. Download the provided service file `gpu-setfan.service` from this repository and enable the service:
  ```sh
  mv gpu-setfan.service /etc/systemd/system/
  systemctl enable gpu-setfan
  systemctl start gpu-setfan
  ```
3. To pass custom arguments, edit `ExecStart` in the service file `gpu-setfan.service` before enabling the service.
   For example, `ExecStart=/usr/local/bin/gpu-setfan --fan-on=60 --min-speed=40 75`.
4. Check if the service is running correctly by inspecting the logs:
   ```sh
   journalctl -u gpu-setfan
   ```

### Windows
The easiest way to run `gpu-setfan` automatically on startup is to use the [nssm](https://nssm.cc/usage) tool to create a windows service.
1. Download the `gpu-setfan` binary and move it to `C:\Program Files\gpu-setfan\gpu-setfan.exe`.
2. Install nssm, for example via winget in the terminal: `winget install nssm`.
3. Create and start the service in the terminal:
   ```sh
    nssm install GpuSetFan "C:\Program Files\gpu-setfan\gpu-setfan.exe"
    nssm start GpuSetFan
   ```
   You can also specify arguments after the path, for example `nssm install GpuSetFan "C:\Program Files\gpu-setfan\gpu-setfan.exe" 75 --fan-on 60`
4. Check if the service is running correctly by inspecting the logs at `C:\ProgramData\gpu-setfan\logs`.
