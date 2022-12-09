// copyright 2022 mikael lund aka wombat
//
// licensed under the apache license, version 2.0 (the "license");
// you may not use this file except in compliance with the license.
// you may obtain a copy of the license at
//
//     http://www.apache.org/licenses/license-2.0
//
// unless required by applicable law or agreed to in writing, software
// distributed under the license is distributed on an "as is" basis,
// without warranties or conditions of any kind, either express or implied.
// see the license for the specific language governing permissions and
// limitations under the license.

//! Matrix65 serial communicator library
//!
//! This contains basic features for serial communication with the
//! MEGA65, as well as auxiliary functions for IO and FileHost access.
//! It is the basis for the CLI tool `matrix65` which is included in
//! this crate.

pub mod filehost;
pub mod io;
pub mod serial;

use anyhow::Result;

/// Load address for Commodore PRG files
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum LoadAddress {
    PET,
    /// Shared for Commodore 64 and Commander X16
    Commodore64,
    /// Shared for Commodore 16, VIC20, and Plus 4
    Commodore16,
    Commodore128,
    Commodore65,
    Custom(u16),
}

#[allow(dead_code)]
impl LoadAddress {
    /// Construct new LoadAddress from 16-bit value
    ///
    /// Example:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// assert_eq!(LoadAddress::new(0x0401), LoadAddress::PET);
    /// assert_eq!(LoadAddress::new(0x0801), LoadAddress::Commodore64);
    /// assert_eq!(LoadAddress::new(0x1001), LoadAddress::Commodore16);
    /// assert_eq!(LoadAddress::new(0x1c01), LoadAddress::Commodore128);
    /// assert_eq!(LoadAddress::new(0x2001), LoadAddress::Commodore65);
    /// assert_eq!(LoadAddress::new(0x1000), LoadAddress::Custom(0x1000));
    /// ~~~
    pub const fn new(address: u16) -> LoadAddress {
        match address {
            0x0401 => LoadAddress::PET,
            0x0801 => LoadAddress::Commodore64,
            0x1001 => LoadAddress::Commodore16,
            0x1c01 => LoadAddress::Commodore128,
            0x2001 => LoadAddress::Commodore65,
            _ => LoadAddress::Custom(address),
        }
    }

    /// Extract load address from first two bytes, little endian.
    ///
    /// Example:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// let bytes: [u8; 3] = [0x01, 0x08, 0xff];
    /// let load_address = LoadAddress::from_bytes(&bytes).unwrap();
    /// assert_eq!(load_address.value(), 0x0801);
    /// ~~~
    pub fn from_bytes(bytes: &[u8]) -> Result<LoadAddress> {
        let address = u16::from_le_bytes(bytes[0..2].try_into()?);
        Ok(Self::new(address))
    }
    /// Returns the 16-bit load address
    ///
    /// Example:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// assert_eq!(LoadAddress::PET.value(), 0x0401);
    /// assert_eq!(LoadAddress::Commodore64.value(), 0x0801);
    /// assert_eq!(LoadAddress::Commodore16.value(), 0x1001);
    /// assert_eq!(LoadAddress::Commodore128.value(), 0x01c01);
    /// assert_eq!(LoadAddress::Commodore65.value(), 0x2001);
    /// assert_eq!(LoadAddress::Custom(0x1000).value(), 0x1000);
    /// ~~~
    pub const fn value(&self) -> u16 {
        match *self {
            LoadAddress::PET => 0x0401,
            LoadAddress::Commodore64 => 0x0801,
            LoadAddress::Commodore16 => 0x1001,
            LoadAddress::Commodore128 => 0x1c01,
            LoadAddress::Commodore65 => 0x2001,
            LoadAddress::Custom(address) => address,
        }
    }
}
