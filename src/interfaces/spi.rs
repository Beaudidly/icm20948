use crate::Error;
use embedded_hal as hal;

use super::SensorInterface;

pub struct SpiInterface<SPI, CS> {
    // SPI port
    spi: SPI,
    // SPI Chip Select pin
    cs: CS,
}

impl<SPI, CS, CommE, PinE> SpiInterface<SPI, CS>
where
    SPI: hal::blocking::spi::Write<u8, Error = CommE>
        + hal::blocking::spi::Transfer<u8, Error = CommE>,
    CS: hal::digital::v2::OutputPin<Error = PinE>,
{
    const DIR_READ: u8 = 0x80; // setting MSB of byte to indicate read

    pub fn new(spi: SPI, cs: CS) -> Result<Self, Error<CommE, PinE>> {
        // TODO init deselect
        let mut device = Self { spi: spi, cs: cs };
        device.cs.set_high().map_err(Error::Pin)?;

        Ok(device)
    }

    fn read_block(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error<CommE, PinE>> {
        buffer[0] = addr | Self::DIR_READ;

        self.cs.set_low().map_err(Error::Pin)?;

        let rc = self.spi.transfer(buffer);

        self.cs.set_high().map_err(Error::Pin)?;
        
        rc.map_err(Error::Comm)?;

        Ok(())
    }

    fn write_block(&mut self, buffer: &[u8]) -> Result<(), Error<CommE, PinE>> {
        self.cs.set_low().map_err(Error::Pin)?;

        let rc =  self.spi.write(buffer);

        self.cs.set_high().map_err(Error::Pin)?;

        rc.map_err(Error::Comm)?;

        Ok(())
    }
}

impl<SPI, CS, CommE, PinE> SensorInterface for SpiInterface<SPI, CS>
where
    SPI: hal::blocking::spi::Write<u8, Error = CommE>
        + hal::blocking::spi::Transfer<u8, Error = CommE>,
    CS: hal::digital::v2::OutputPin<Error = PinE>,
{
    type InterfaceError = Error<CommE, PinE>;

    fn register_read(&mut self, addr: u8) -> Result<u8, Self::InterfaceError> {
        let mut buffer: [u8; 2] = [0; 2];
        self.read_block(addr, &mut buffer)?;

        Ok(buffer[1])
    }

    fn register_write(&mut self, addr: u8, val: u8) -> Result<(), Self::InterfaceError> {
        let buffer: [u8; 2] = [addr, val];
        self.write_block(&buffer)?;
        Ok(())
    }

    fn read_vec3_i16(&mut self, reg: u8) -> Result<[i16; 3], Self::InterfaceError> {
        Ok([0, 0, 0])
    }
}
