#![no_std]

use embedded_hal as hal;
use hal::blocking::delay::DelayMs;

mod interfaces;
use interfaces::{SensorInterface, SpiInterface};

#[derive(Debug)]
pub enum Error<CommE, PinE> {
    Comm(CommE),
    Pin(PinE),
    Unresponsive,
    InvalidID,
}

pub enum Bank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
}

impl Bank {
    fn get_num(&self) -> u8 {
        match self {
            Bank::Bank0 => 0,
            Bank::Bank1 => 1,
            Bank::Bank2 => 2,
            Bank::Bank3 => 3,
        }
    }
}

pub struct ICM20948<SI> {
    si: SI,
    bank: Bank,
}

//pub mod builder {
pub fn new_spi<SPI, CS, CommE, PinE>(
    spi: SPI,
    cs: CS,
) -> Result<ICM20948<SpiInterface<SPI, CS>>, Error<CommE, PinE>>
where
    SPI: hal::blocking::spi::Write<u8, Error = CommE>
        + hal::blocking::spi::Transfer<u8, Error = CommE>,
    CS: hal::digital::v2::OutputPin<Error = PinE>,
{
    let interface = interfaces::SpiInterface::new(spi, cs)?;
    ICM20948::new_with_interface(interface)
}
//}

impl<SI, CommE, PinE> ICM20948<SI>
where
    SI: SensorInterface<InterfaceError = Error<CommE, PinE>>,
{
    pub fn new_with_interface(sensor_interface: SI) -> Result<Self, SI::InterfaceError> {
        let mut instance = Self {
            si: sensor_interface,
            bank: Bank::Bank0,
        };

        instance.change_bank(Bank::Bank0)?;

        Ok(instance)
    }

    pub fn soft_reset(
        &mut self,
        delay_source: &mut impl DelayMs<u8>,
    )  -> Result<(), SI::InterfaceError> {

        // Reset power
        const PWR_RESET: u8 = 1 << 7;
        self.si.register_write(REG_PWR_MGMT_1, PWR_RESET)?;

        // power can take 100ms to reset, so wait longer
        let mut reset_success = false;
        for _ in 0..10 {
            //The reset bit automatically clears to 0 once the reset is done.
            if let Ok(reg_val) = self.si.register_read(REG_PWR_MGMT_1) {
                if reg_val & PWR_RESET == 0 {
                    reset_success = true;
                    break;
                }
            }
            delay_source.delay_ms(10);
        }
        if !reset_success {
            #[cfg(feature = "rttdebug")]
            rprintln!("couldn't read REG_PWR_MGMT_1");
            return Err(Error::Unresponsive);
        }
        
        Ok(())
    }

    pub fn new_setup(&mut self, delay_source: &mut impl DelayMs<u8>) -> Result<(), SI::InterfaceError> {

        if self.who_am_i()? != WHO_AM_I {
            return Err(Error::InvalidID);
        }

        self.soft_reset(delay_source)?;

        // turn off sleep and set clock to auto select
        self.change_bank(Bank::Bank0)?;
        self.si.register_write(REG_PWR_MGMT_1, 0x01)?;

        // set sample mode
        self.change_bank(Bank::Bank0)?;
        self.si.register_write(REG_LP_CONFIG, (1 << 5) | (1 << 4))?; 

        // set full scale for accelerometer/gyro
        //self.change_bank(Bank::Bank2)?;

        //self.si.register_write(0

        Ok(())
    }

    pub fn setup(&mut self) -> Result<(), SI::InterfaceError> {
        const CLK_SEL_AUTO: u8 = 0x01;
        const SENSOR_ENABLE_ALL: u8 = 0x00;

        // set sample mode
        self.si.register_write(REG_LP_CONFIG, 0x30)?;


        self.si.register_write(REG_INT_ENABLE, 0x0)?;
        self.si.register_write(REG_INT_ENABLE_1, 0x0)?;
        self.si.register_write(REG_INT_ENABLE_2, 0x0)?;
        self.si.register_write(REG_INT_ENABLE_3, 0x0)?;
        self.si.register_write(REG_FIFO_EN_2, 0b11111)?;

        self.si.register_write(REG_PWR_MGMT_1, (1 << 6) | 0x01)?;

        Ok(())
    }

    pub fn who_am_i(&mut self) -> Result<u8, SI::InterfaceError> {
        self.change_bank(Bank::Bank0)?;
        self.si.register_read(0x0)
    }

    pub fn change_bank(&mut self, bank: Bank) -> Result<(), SI::InterfaceError> {
        self.si.register_write(REG_BANK_SEL, bank.get_num())?;
        self.bank = bank;
        Ok(())
    }

    pub fn get_temp(&mut self) -> Result<f32, SI::InterfaceError> {
        self.change_bank(Bank::Bank0)?;

        let low = self.si.register_read(REG_TEMP_OUT_L)?;
        let high = self.si.register_read(REG_TEMP_OUT_H)?;

        let temp_out = ((high as u16) << 8) | (low as u16);

        let deg_c = (((temp_out as f32) - 20.0) / 333.87) + 21.0;

        Ok(deg_c)
    }

    pub fn get_raw_gyro(&mut self) -> Result<[i16; 3], SI::InterfaceError> {
        self.si.read_vec3_i16(REG_GYRO)
    }

    pub fn get_raw_accel(&mut self) -> Result<[i16; 3], SI::InterfaceError> {
        self.si.read_vec3_i16(REG_ACCEL)
    }

    pub fn dev_read(&mut self, addr: u8) -> Result<u8, SI::InterfaceError> {
        self.si.register_read(addr)
    }
}


const WHO_AM_I: u8 = 0xea;

const REG_BANK_SEL: u8 = 0x7f;
const REG_GYRO: u8 = 0x33;
const REG_ACCEL: u8 = 0x2d;
const REG_TEMP_OUT_H: u8 = 0x39;
const REG_TEMP_OUT_L: u8 = 0x3A;

const REG_FIFO_EN_2: u8 = 0x67;
const REG_INT_ENABLE: u8 = 0x10;
const REG_INT_ENABLE_1: u8 = 0x11;
const REG_INT_ENABLE_2: u8 = 0x12;
const REG_INT_ENABLE_3: u8 = 0x13;
const REG_PWR_MGMT_1: u8 = 0x6;
const REG_LP_CONFIG: u8 = 0x05;
const REG_ACCEL_CONFIG: u8 = 0x14;
