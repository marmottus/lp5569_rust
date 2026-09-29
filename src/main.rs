use i2c::lp5569::{Led, Lp5569, Rgb};

use serde::Deserialize;

use docopt::Docopt;
use std::env::args;

pub mod i2c;

const USAGE: &str = "
Control the LP5569 LED driver

Usage:
  lp5569 [options]
  lp5569 [options] rgb <led> <red> <green> <blue>
  lp5569 pulse <led> <red> <green> <blue> <pwm> <ramp>
  lp5569 (-h | --help)
  lp5569 (-v | --version)

Options:
  -b --bus BUS               Specify the I2C bus to use [default: 1].
  -i --i2c-address=<addr>    I2C slave address [default: 50].
  -r --reset                 Reset the LP5569 device before performing any operations.
  -h --help                  Show this help text.
  --version                  Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_rgb: bool,
    cmd_pulse: bool,
    arg_led: u8,
    arg_red: u8,
    arg_green: u8,
    arg_blue: u8,
    arg_pwm: u8,
    arg_ramp: u32,
    flag_i2c_address: u16,
    flag_bus: u8,
    flag_reset: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| {
            d.argv(args())
                .version(Some(env!("CARGO_PKG_VERSION").to_string()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());

    if !args.cmd_rgb && !args.flag_reset && !args.cmd_pulse {
        println!("{}", USAGE);
        return;
    }

    let i2c_bus = format!("/dev/i2c-{}", args.flag_bus);

    let mut lp5569 = match Lp5569::new(i2c_bus, args.flag_i2c_address) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("could not open i2c device: {:?}", err);
            std::process::exit(1);
        }
    };

    if args.flag_reset {
        match lp5569.reset() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to reset LP5569 device: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    if args.cmd_rgb {
        let led = match args.arg_led {
            0 => Led::Led0,
            1 => Led::Led1,
            2 => Led::Led2,
            led => {
                eprintln!("Invalid LED {led}, expected 0, 1 or 2");
                std::process::exit(1);
            }
        };

        let led_color = Rgb {
            red: args.arg_red,
            green: args.arg_green,
            blue: args.arg_blue,
        };

        match lp5569.set_rgb(&led, led_color) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to set RGB color: {:?}", err);
                std::process::exit(1);
            }
        }
    } else if args.cmd_pulse {
        let led = match args.arg_led {
            0 => Led::Led0,
            1 => Led::Led1,
            2 => Led::Led2,
            led => {
                eprintln!("Invalid LED {led}, expected 0, 1 or 2");
                std::process::exit(1);
            }
        };

        match lp5569.set_pulse(
            &led,
            args.arg_red != 0,
            args.arg_green != 0,
            args.arg_blue != 0,
            args.arg_pwm,
            args.arg_ramp,
        ) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to set pulse: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
