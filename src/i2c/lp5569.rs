extern crate i2cdev;
use std::fmt;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use std::path::Path;

#[derive(Debug)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.red, self.green, self.blue)
    }
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum Led {
    Led0 = 0,
    Led1 = 1,
    Led2 = 2,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum Lp5569Reg {
    Config = 0x00,
    LedEngineControl1 = 0x01,
    LedEngineControl2 = 0x02,
    Led0Control = 0x07,
    Led1Control = 0x08,
    Led2Control = 0x09,
    Led3Control = 0x0A,
    Led4Control = 0x0B,
    Led5Control = 0x0C,
    Led6Control = 0x0D,
    Led7Control = 0x0E,
    Led8Control = 0x0F,
    Led0Pwm = 0x16,
    Led1Pwm = 0x17,
    Led2Pwm = 0x18,
    Led3Pwm = 0x19,
    Led4Pwm = 0x1A,
    Led5Pwm = 0x1B,
    Led6Pwm = 0x1C,
    Led7Pwm = 0x1D,
    Led8Pwm = 0x1E,
    Led0Current = 0x22,
    Led1Current = 0x23,
    Led2Current = 0x24,
    Led3Current = 0x25,
    Led4Current = 0x26,
    Led5Current = 0x27,
    Led6Current = 0x28,
    Led7Current = 0x29,
    Led8Current = 0x2A,
    Misc = 0x2F,
    Engine1Pc = 0x30,
    Engine2Pc = 0x31,
    Engine3Pc = 0x32,
    Misc2 = 0x33,
    EngineStatus = 0x3C,
    IoControl = 0x3D,
    VariableD = 0x3E,
    Reset = 0x3F,
    Engine1VariableA = 0x42,
    Engine2VariableA = 0x43,
    Engine3VariableA = 0x44,
    MasterFader1 = 0x46,
    MasterFader2 = 0x47,
    MasterFader3 = 0x48,
    MasterFaderPwm = 0x4A,
    Engine1ProgStart = 0x4B,
    Engine2ProgStart = 0x4C,
    Engine3ProgStart = 0x4D,
    ProgMemPageSelect = 0x4F,
    ProgramMem00 = 0x50,
    ProgramMem01 = 0x51,
    ProgramMem02 = 0x52,
    ProgramMem03 = 0x53,
    ProgramMem04 = 0x54,
    ProgramMem05 = 0x55,
    ProgramMem06 = 0x56,
    ProgramMem07 = 0x57,
    ProgramMem08 = 0x58,
    ProgramMem09 = 0x59,
    ProgramMem10 = 0x5A,
    ProgramMem11 = 0x5B,
    ProgramMem12 = 0x5C,
    ProgramMem13 = 0x5D,
    ProgramMem14 = 0x5E,
    ProgramMem15 = 0x5F,
    ProgramMem16 = 0x60,
    ProgramMem17 = 0x61,
    ProgramMem18 = 0x62,
    ProgramMem19 = 0x63,
    ProgramMem20 = 0x64,
    ProgramMem21 = 0x65,
    ProgramMem22 = 0x66,
    ProgramMem23 = 0x67,
    ProgramMem24 = 0x68,
    ProgramMem25 = 0x69,
    ProgramMem26 = 0x6A,
    ProgramMem27 = 0x6B,
    ProgramMem28 = 0x6C,
    ProgramMem29 = 0x6D,
    ProgramMem30 = 0x6E,
    ProgramMem31 = 0x6F,
    Engine1Mapping1 = 0x70,
    Engine1Mapping2 = 0x71,
    Engine2Mapping1 = 0x72,
    Engine2Mapping2 = 0x73,
    Engine3Mapping1 = 0x74,
    Engine3Mapping2 = 0x75,
    PwmConfig = 0x80,
    LedFault1 = 0x81,
    LedFault2 = 0x82,
    GeneralFault = 0x83,
}

impl fmt::Display for Lp5569Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({:#04x})", self, *self as u8)
    }
}

const REG_CONFIG_ENABLE: u8 = 1 << 6;
/*
 * 1: Internal 32-kHz oscillator
 * 2-3: auto mode
 * 6: auto increment i2c address
 */
const REG_MISC_DEFAULT: u8 = 1 << 0 | 3 << 3 | 1 << 6;

const LED_RED_CURRENT: u8 = 10; // 1 mA
const LED_GREEN_CURRENT: u8 = 3; // 0.3 mA
const LED_BLUE_CURRENT: u8 = 8; // 0.8 mA

pub const LP5569_SLAVE_ADDR: u16 = 0x32;

pub struct Lp5569 {
    device: LinuxI2CDevice,
}

impl Lp5569 {
    pub fn new<P: AsRef<Path>>(path: P, i2c_address: u16) -> Result<Lp5569, LinuxI2CError> {
        println!("Open i2c device at {} address {:#04x}", path.as_ref().display(), i2c_address);
        let device = LinuxI2CDevice::new(path, i2c_address)?;
        let mut lp5569 = Lp5569 { device };
        lp5569.reset()?;

        Ok(lp5569)
    }

    pub fn set_rgb(&mut self, led: Led, color: Rgb) -> Result<(), LinuxI2CError> {
        let reg = match led {
            Led::Led0 => Lp5569Reg::Led0Pwm,
            Led::Led1 => Lp5569Reg::Led3Pwm,
            Led::Led2 => Lp5569Reg::Led6Pwm,
        };

        println!("Setting RGB for LED {:?} to color {}", led, color);

        self.write_bytes(reg, &[color.red, color.green, color.blue])?;

        Ok(())
    }

    fn write_byte(&mut self, reg: Lp5569Reg, value: u8) -> Result<(), LinuxI2CError> {
        println!("write byte to reg {}", reg);
        match self.device.smbus_write_byte_data(reg as u8, value) {
            Ok(_) => Ok(()),
            Err(err) => {
				eprintln!("Failed to write byte to reg {}: {:?}", reg, err);
				Err(err)
			}
        }
    }

    fn write_bytes(&mut self, reg: Lp5569Reg, values: &[u8]) -> Result<(), LinuxI2CError> {
        println!("write {} bytes to reg {}", values.len(), reg);
        match self.device.smbus_write_i2c_block_data(reg as u8, values) {
            Ok(_) => Ok(()),
            Err(err) => {
				eprintln!("Failed to write bytes to reg {}: {:?}", reg, err);
				Err(err)
			}
        }
    }

    fn reset(&mut self) -> Result<(), LinuxI2CError> {
        self.write_byte(Lp5569Reg::Reset, 0xFF)?;
        self.write_byte(Lp5569Reg::Misc, REG_MISC_DEFAULT)?;
        self.write_byte(Lp5569Reg::Config, REG_CONFIG_ENABLE)?;

        self.write_byte(Lp5569Reg::Led0Current, LED_RED_CURRENT)?;
        self.write_byte(Lp5569Reg::Led3Current, LED_RED_CURRENT)?;
        self.write_byte(Lp5569Reg::Led6Current, LED_RED_CURRENT)?;

        self.write_byte(Lp5569Reg::Led1Current, LED_GREEN_CURRENT)?;
        self.write_byte(Lp5569Reg::Led4Current, LED_GREEN_CURRENT)?;
        self.write_byte(Lp5569Reg::Led7Current, LED_GREEN_CURRENT)?;

        self.write_byte(Lp5569Reg::Led2Current, LED_BLUE_CURRENT)?;
        self.write_byte(Lp5569Reg::Led5Current, LED_BLUE_CURRENT)?;
        self.write_byte(Lp5569Reg::Led8Current, LED_BLUE_CURRENT)?;

        Ok(())
    }
}
