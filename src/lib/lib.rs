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
