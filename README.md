# gpu-setfan

Controls GPU fan speed to maintain a target temperature.
Instead of providing a fan curve, like many other tools require, you only specify the temperature at which the GPU should run under load.
`gpu-setfan` automatically finds the smallest fan speed to maintain that target temperature, which minimizes the fan noise.

Supports multiple GPUs: controls the fan speed of each detected GPU separately.

**Requirements**: At the moment only **Nvidia** GPUs are supported.

## How it works
Internally, a [PI-controller](https://en.wikipedia.org/wiki/Proportional%E2%80%93integral%E2%80%93derivative_controller)
is used to find the control (i.e., the fan speed) that steers the system to the so-called *setpoint* (i.e., the target temperature).

The general behavior is as follows:
- At the start, the fans are off (speed = 0%).
- The fans get turned on when the temperature exceeds the `--fan-on` temperature.
- Then the fan speed is controlled with a PI controller to maintain the target temperature. The fan speed is always larger than `--min-speed` to ensure stable operation.
- When the temperature falls below the `--fan-off` temperature, the fans get turned off again.

Because the `--fan-off` temperature is smaller than the `--fan-on` temperature, frequent on/off cycling is prevented (hysteresis).

## Usage

```
gpu-setfan [OPTIONS] [TARGET_TEMPERATURE]
```

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
