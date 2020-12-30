#![no_std]

use embedded_hal as hal;

mod interfaces;
use interfaces::{SensorInterface, SpiInterface};

#[derive(Debug)]
pub enum Error<CommE, PinE> {
    Comm(CommE),
    Pin(PinE),
}

pub enum Bank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
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

    pub fn who_am_i(&mut self) -> Result<u8, SI::InterfaceError> {
        self.change_bank(Bank::Bank0)?;
        self.si.register_read(0x0)
    }

    pub fn change_bank(&mut self, bank: Bank) -> Result<(), SI::InterfaceError> {
        match bank {
            Bank::Bank0 => self.si.register_write(REG_BANK_SEL, 0),
            Bank::Bank1 => self.si.register_write(REG_BANK_SEL, 1),
            Bank::Bank2 => self.si.register_write(REG_BANK_SEL, 2),
            Bank::Bank3 => self.si.register_write(REG_BANK_SEL, 3),
        }
        .and_then(|_: ()| -> Result<(), SI::InterfaceError> {
            self.bank = bank;
            Ok(())
        })
    }
}

const REG_BANK_SEL: u8 = 0x7f;
