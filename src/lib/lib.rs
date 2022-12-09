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
use std::convert::From;
use std::fmt;

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
    /// Shared for Commodore 65 and MEGA65
    Commodore65,
    Custom(u16),
}

#[allow(dead_code)]
impl LoadAddress {
    /// Construct new LoadAddress from 16-bit value
    ///
    /// Examples:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// assert_eq!(LoadAddress::new(0x0401), LoadAddress::PET);
    /// assert_eq!(LoadAddress::new(0x0801), LoadAddress::Commodore64);
    /// assert_eq!(LoadAddress::new(0x1001), LoadAddress::Commodore16);
    /// assert_eq!(LoadAddress::new(0x1c01), LoadAddress::Commodore128);
    /// assert_eq!(LoadAddress::new(0x2001), LoadAddress::Commodore65);
    /// assert_eq!(LoadAddress::new(0xc000), LoadAddress::Custom(0xc000));
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
    /// Examples:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// let bytes: [u8; 3] = [0x01, 0x08, 0xff];
    /// let load_address = LoadAddress::from_bytes(&bytes).unwrap();
    /// assert_eq!(load_address, LoadAddress::Commodore64);
    /// assert_eq!(load_address.value(), 0x0801);
    /// ~~~
    pub fn from_bytes(bytes: &[u8]) -> Result<LoadAddress> {
        let address = u16::from_le_bytes(bytes[0..2].try_into()?);
        Ok(Self::new(address))
    }
    /// Returns the 16-bit load address
    ///
    /// Examples:
    /// ~~~
    /// use matrix65::LoadAddress;
    /// assert_eq!(LoadAddress::PET.value(), 0x0401);
    /// assert_eq!(LoadAddress::Commodore64.value(), 0x0801);
    /// assert_eq!(LoadAddress::Commodore16.value(), 0x1001);
    /// assert_eq!(LoadAddress::Commodore128.value(), 0x01c01);
    /// assert_eq!(LoadAddress::Commodore65.value(), 0x2001);
    /// assert_eq!(LoadAddress::Custom(0xc000).value(), 0xc000);
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

/// Examples:
/// ~~~
/// use matrix65::LoadAddress;
/// assert_eq!(u16::from(LoadAddress::Commodore64), 0x0801);
/// let value: u16 = LoadAddress::Commodore64.into();
/// assert_eq!(value, 0x0801);
/// ~~~
impl From<LoadAddress> for u16 {
    fn from(load_address: LoadAddress) -> u16 {
        load_address.value()
    }
}

/// Examples:
/// ~~~
/// use matrix65::LoadAddress;
/// assert_eq!(LoadAddress::from(0x0401), LoadAddress::PET);
/// assert_eq!(LoadAddress::from(0x0801), LoadAddress::Commodore64);
/// assert_eq!(LoadAddress::from(0x1001), LoadAddress::Commodore16);
/// assert_eq!(LoadAddress::from(0x1c01), LoadAddress::Commodore128);
/// assert_eq!(LoadAddress::from(0x2001), LoadAddress::Commodore65);
/// assert_eq!(LoadAddress::from(0xc000), LoadAddress::Custom(0xc000));
/// 
/// let address: LoadAddress = 0x0801.into();
/// assert_eq!(address, LoadAddress::Commodore64);
/// ~~~
impl From<u16> for LoadAddress {
    fn from(address: u16) -> Self {
        LoadAddress::new(address)
    }
}

impl fmt::Display for LoadAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.value())
    }
}
