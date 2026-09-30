use std::fmt;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

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

const LED_ENGINE_CONTROL1_FREE_RUN: u8 = 0b10;

const LED_ENGINE_CONTROL2_LOAD_PROGRAM: u8 = 0b01;
const LED_ENGINE_CONTROL2_RUN_PROGRAM: u8 = 0b10;

const LED_ENGINE1_CONTROL_SHIFT: u8 = 6;
const LED_ENGINE2_CONTROL_SHIFT: u8 = 4;
const LED_ENGINE3_CONTROL_SHIFT: u8 = 2;

const ENGINE_BUSY_MASK: u8 = 0b0010_0000;

const PRESCALE0_CLK: u16 = 32768 / 16;
const PRESCALE1_CLK: u16 = 32768 / 512;

const PRESCALE0: f32 = 1. / (PRESCALE0_CLK as f32);
const PRESCALE1: f32 = 1. / (PRESCALE1_CLK as f32);

// 1 0 0 1 1 1 1 1 1 SRAM_ADDR[6-0]
const MAP_ADDR_OPCODE: u16 = 0b10011111_10000000;

// 0 1 0 0 0 0 0 0 PWM[7-0]
const SET_PWM_OPCODE: u16 = 0b01000000_00000000;

// 1 0 1 LOOP_CNT[12-7] STEP_NUM[6-0]
// Step number is relative to the program start address.
const BRANCH_OPCODE: u16 = 0b10100000_00000000;

// 1 1 0 int[12] reset[11] 0 0 0 0 0 0 0 0 0 0 0
const END_OPCODE: u16 = 0b11000000_00000000;

// 0 PRESCALE[14] STEP_TIME[13-9] SIGN[8] NUM_INC[7-0]
const RAMP_OPCODE: u16 = 0b00000000_00000000;
const RAMP_NEGATIVE_MASK: u16 = 1 << 8;

// 0 PRESCALE[14]
const WAIT_OPCODE: u16 = 0b00000000_00000000;

const PRESCALE_MASK: u16 = 1 << 14;
const TIME_SHIFT: u8 = 9;

// LP5569 has 16-instructions of 2 bytes per page in its SRAM.
const PAGE_SIZE: u8 = 16;

#[derive(Debug)]
pub enum Lp5569Error {
    Error(&'static str),
    LinuxI2CError(LinuxI2CError),
}

impl From<LinuxI2CError> for Lp5569Error {
    fn from(err: LinuxI2CError) -> Self {
        Lp5569Error::LinuxI2CError(err)
    }
}

fn get_prescaled_cmd(time: f32) -> Result<u16, Lp5569Error> {
    if time < PRESCALE0 {
        return Err(Lp5569Error::Error("Time is below minimum prescale value"));
    }

    if time <= 31. * PRESCALE0 {
        let time_prescaled = (time * PRESCALE0_CLK as f32).round();
        return Ok((time_prescaled as u16) << TIME_SHIFT);
    }

    if time <= 31. * PRESCALE1 {
        let time_prescaled = (time * PRESCALE1_CLK as f32).round();
        return Ok(PRESCALE_MASK | ((time_prescaled as u16) << TIME_SHIFT));
    }

    Err(Lp5569Error::Error("Time exceeds maximum prescale value"))
}

fn cmd_wait(time: f32) -> Result<u16, Lp5569Error> {
    Ok(get_prescaled_cmd(time)? | WAIT_OPCODE)
}

fn cmd_ramp(time: f32, pwm: i16) -> Result<u16, Lp5569Error> {
    let step_time = time / (pwm.abs() as f32);
    let prescaled_cmd = get_prescaled_cmd(step_time)?;

    if pwm < 0 {
        return Ok(RAMP_OPCODE | prescaled_cmd | RAMP_NEGATIVE_MASK | pwm.abs() as u16);
    }

    Ok(RAMP_OPCODE | prescaled_cmd | pwm as u16)
}

pub struct Lp5569 {
    device: LinuxI2CDevice,
}

impl Lp5569 {
    pub fn new<P: AsRef<Path>>(path: P, i2c_address: u16) -> Result<Lp5569, LinuxI2CError> {
        println!(
            "Open i2c device at {} address {:#04x}",
            path.as_ref().display(),
            i2c_address
        );
        let device = LinuxI2CDevice::new(path, i2c_address)?;
        let lp5569 = Lp5569 { device };

        Ok(lp5569)
    }

    pub fn set_rgb(&mut self, led: &Led, color: Rgb) -> Result<(), LinuxI2CError> {
        let engine_ctrl_shift: u8;
        let led_pwm_reg: Lp5569Reg;

        match led {
            Led::Led0 => {
                engine_ctrl_shift = LED_ENGINE1_CONTROL_SHIFT;
                led_pwm_reg = Lp5569Reg::Led0Pwm;
            }
            Led::Led1 => {
                engine_ctrl_shift = LED_ENGINE2_CONTROL_SHIFT;
                led_pwm_reg = Lp5569Reg::Led3Pwm;
            }
            Led::Led2 => {
                engine_ctrl_shift = LED_ENGINE3_CONTROL_SHIFT;
                led_pwm_reg = Lp5569Reg::Led6Pwm;
            }
        };

        let mut engine_ctrl_2 = self.read_byte(Lp5569Reg::LedEngineControl2)?;
        engine_ctrl_2 &= !(0b11 << engine_ctrl_shift);

        self.write_byte(Lp5569Reg::LedEngineControl2, engine_ctrl_2)?;

        println!("Setting RGB for LED {:?} to color {}", led, color);

        self.write_bytes(led_pwm_reg, &[color.red, color.green, color.blue])?;

        Ok(())
    }

    pub fn set_pulse(
        &mut self,
        led: &Led,
        red: bool,
        green: bool,
        blue: bool,
        pwm: u8,
        ramp: u32,
    ) -> Result<(), Lp5569Error> {
        let page: u8;
        let engine_reg: Lp5569Reg;
        let engine_ctrl_shift: u8;

        self.set_rgb(
            led,
            Rgb {
                red: 0,
                green: 0,
                blue: 0,
            },
        )?;

        match led {
            Led::Led0 => {
                engine_ctrl_shift = LED_ENGINE1_CONTROL_SHIFT;
                page = 0;
                engine_reg = Lp5569Reg::Engine1ProgStart;
            }
            Led::Led1 => {
                engine_ctrl_shift = LED_ENGINE2_CONTROL_SHIFT;
                page = 1;
                engine_reg = Lp5569Reg::Engine2ProgStart;
            }
            Led::Led2 => {
                engine_ctrl_shift = LED_ENGINE3_CONTROL_SHIFT;
                page = 2;
                engine_reg = Lp5569Reg::Engine3ProgStart;
            }
        };

        let mut engine_ctrl_2 = self.read_byte(Lp5569Reg::LedEngineControl2)?;

        // Disable all engines
        self.write_byte(Lp5569Reg::LedEngineControl2, 0x00)?;

        self.write_byte(
            Lp5569Reg::LedEngineControl2,
            LED_ENGINE_CONTROL2_LOAD_PROGRAM << engine_ctrl_shift,
        )?;
        self.wait_for_engine()?;

        let wait_time: f32 = 0.3;
        let ramp_time_sec = ramp as f32 / 1000.;

        println!(
            "Set pulse with PWM {} ramp time {} ms and wait time {} ms",
            pwm,
            ramp,
            wait_time * 1000.
        );

        let ramp_inc_opcode = cmd_ramp(ramp_time_sec, pwm as i16)?;
        let ramp_dec_opcode = cmd_ramp(ramp_time_sec, -(pwm as i16))?;
        let sleep_opcode = cmd_wait(wait_time)?;

        let mut led_map_table: u16 = (if red { 1 << 0 } else { 0 })
            | (if green { 1 << 1 } else { 0 })
            | (if blue { 1 << 2 } else { 0 });

        match led {
            Led::Led0 => (),
            Led::Led1 => led_map_table <<= 3,
            Led::Led2 => led_map_table <<= 6,
        };

        let led_map_addr = page * PAGE_SIZE;

        #[rustfmt::skip]
        let program: [u8; 16] = [
            // 00: set led map table
            (led_map_table >> 8) as u8, led_map_table as u8,
            // 01:  Program start map_addr map_table_addr
            (MAP_ADDR_OPCODE >> 8) as u8, led_map_addr | MAP_ADDR_OPCODE as u8,
            // 02: set_pwm 0
            (SET_PWM_OPCODE >> 8) as u8, 0x00,
            // 03: ramp time, pwm
            (ramp_inc_opcode >> 8) as u8, ramp_inc_opcode as u8,
            // 04: ramp time, -pwm
            (ramp_dec_opcode >> 8) as u8, ramp_dec_opcode as u8,
            // 05: wait time
            (sleep_opcode >> 8) as u8, sleep_opcode as u8,
            (BRANCH_OPCODE >> 8) as u8, 0x02, // 06: branch 0, 0x02
            (END_OPCODE >> 8) as u8, END_OPCODE as u8, // 07: end
        ];

        let engine_prog_start = led_map_addr + 0x1;

        self.write_byte(Lp5569Reg::ProgMemPageSelect, page as u8)?;
        self.write_bytes(Lp5569Reg::ProgramMem00, &program)?;

        self.write_byte(engine_reg, engine_prog_start)?;

        engine_ctrl_2 &= !(0b11 << engine_ctrl_shift);
        engine_ctrl_2 |= LED_ENGINE_CONTROL2_RUN_PROGRAM << engine_ctrl_shift;
        self.write_byte(Lp5569Reg::LedEngineControl2, engine_ctrl_2)?;

        let mut engine_ctrl_1 = self.read_byte(Lp5569Reg::LedEngineControl1)?;
        engine_ctrl_1 &= !(0b11 << engine_ctrl_shift);
        engine_ctrl_1 |= LED_ENGINE_CONTROL1_FREE_RUN << engine_ctrl_shift;

        self.write_byte(Lp5569Reg::LedEngineControl1, engine_ctrl_1)?;

        Ok(())
    }

    fn wait_for_engine(&mut self) -> Result<(), LinuxI2CError> {
        loop {
            let engine_status = self.read_byte(Lp5569Reg::EngineStatus)?;
            if (engine_status & ENGINE_BUSY_MASK) == 0 {
                break;
            }

            sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn read_byte(&mut self, reg: Lp5569Reg) -> Result<u8, LinuxI2CError> {
        println!("Reading byte from reg {}", reg);
        match self.device.smbus_read_byte_data(reg as u8) {
            Ok(value) => {
                println!("Read {:02X} from reg {}", value, reg);
                Ok(value)
            }
            Err(err) => {
                eprintln!("Failed to read byte from reg {}: {:?}", reg, err);
                Err(err)
            }
        }
    }

    fn write_byte(&mut self, reg: Lp5569Reg, value: u8) -> Result<(), LinuxI2CError> {
        println!("write {:02X} to reg {}", value, reg);
        match self.device.smbus_write_byte_data(reg as u8, value) {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("Failed to write byte to reg {}: {:?}", reg, err);
                Err(err)
            }
        }
    }

    fn write_bytes(&mut self, reg: Lp5569Reg, values: &[u8]) -> Result<(), LinuxI2CError> {
        println!("write {:02X?} to reg {}", values, reg);
        match self.device.smbus_write_i2c_block_data(reg as u8, values) {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("Failed to write bytes to reg {}: {:?}", reg, err);
                Err(err)
            }
        }
    }

    pub fn reset(&mut self) -> Result<(), LinuxI2CError> {
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
