//! A USB driver for i.MX RT processors
//!
//! `imxrt-usbd` provides a [`usb-device`] USB bus implementation, allowing you
//! to add USB device features to your embedded Rust program. See each module
//! for usage and examples.
//!
//! To interface the library, you must define a safe implementation of [`Peripherals`].
//! See the trait documentation for more information.
//!
//! # General guidance
//!
//! The driver does not configure any of the CCM or CCM_ANALOG registers. You are
//! responsible for configuring these peripherals for proper USB functionality. See
//! the `imxrt-usbd` hardware examples to see different ways of configuring PLLs and
//! clocks.
//!
//! [`usb-device`]: https://crates.io/crates/usb-device
//!
//! # Debugging features
//!
//! Enable the `defmt-03` feature to activate internal logging using defmt (version 0.3).
//!
//! # `imxrt-ral` compatibility
//!
//! Enable the `imxrt-ral` feature to add a [`Peripherals`] implementation for imxrt-ral
//! instances. Note that you, or something in your dependency hierarchy, will also need
//! to enable an `imxrt-ral` feature; otherwise, this package may not build.

#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

#[macro_use]
mod log;

mod buffer;
mod bus;
mod cache;
mod driver;
mod endpoint;
mod qh;
mod ral;
mod state;
mod td;
mod vcell;

pub use buffer::EndpointMemory;
pub use bus::{BusAdapter, Speed};
pub mod gpt;
pub use state::{EndpointState, MAX_ENDPOINTS};

/// A type that owns all USB register blocks
///
/// An implementation of `Peripherals` is expected to own the USB1
/// or USB2 registers. This includes
///
/// - USB core registers
/// - USB PHY registers
///
/// When an instance of `Peripherals` exists, you must make sure that
/// nothing else, other than the owner of the `Peripherals` object,
/// accesses those registers.
///
/// # Safety
///
/// `Peripherals` should only be implemented on a type that
/// owns the various register blocks required for all USB
/// operation. Incorrect usage, or failure to ensure exclusive
/// ownership, could lead to data races and incorrect USB functionality.
///
/// All pointers are expected to point at the starting register block
/// for the specified peripheral. Calls to the functions must return the
/// the same value every time they're called.
///
/// # Example
///
/// When you enable this package's `imxrt-ral` feature, the package
/// provides a `Peripherals` implementation for imxrt-ral instances.
/// See the package source for an example of a `Peripherals` implementation
///
/// You'll _use_ this imxrt-ral implementation as follows:
///
/// ```no_run
/// use imxrt_ral as ral;
/// use imxrt_usbd::{Instances, BusAdapter};
///
/// let instances = Instances {
///     usb: unsafe { ral::usb::USB::instance() },
///     usbnc: unsafe { ral::usbnc::USBNC::instance() },
///     usbphy: unsafe { ral::usbphy::USBPHY::instance() },
/// };
///
/// # static EP_MEMORY: imxrt_usbd::EndpointMemory<1024> = imxrt_usbd::EndpointMemory::new();
/// # static EP_STATE: imxrt_usbd::EndpointState = imxrt_usbd::EndpointState::max_endpoints();
/// let bus_adapter = BusAdapter::new(
///     instances,
///     &EP_MEMORY,
///     &EP_STATE,
/// );
/// ```
pub unsafe trait Peripherals {
    /// Returns the pointer to the USB register block.
    fn usb(&self) -> *const ();
    /// Returns the pointer to the USBPHY register block.
    fn usbphy(&self) -> *const ();
}

#[cfg(feature = "imxrt-ral")]
mod ral_compat {
    use imxrt_ral as ral;

    /// Aggregate of `imxrt-ral` USB peripheral instances.
    ///
    /// This takes ownership of USB peripheral instances and implements
    /// [`Peripherals`](super::Peripherals). You can use this type to allocate a USB device
    /// driver.
    pub struct Instances<const N: u8> {
        /// USB core registers.
        pub usb: ral::usb::Instance<N>,
        /// USB non-core registers.
        pub usbnc: ral::usbnc::Instance<N>,
        /// USBPHY registers.
        pub usbphy: ral::usbphy::Instance<N>,
    }

    unsafe impl<const N: u8> super::Peripherals for Instances<N> {
        fn usb(&self) -> *const () {
            (&*self.usb as *const ral::usb::RegisterBlock).cast()
        }
        fn usbphy(&self) -> *const () {
            (&*self.usbphy as *const ral::usbphy::RegisterBlock).cast()
        }
    }
}

#[cfg(feature = "imxrt-ral")]
pub use ral_compat::*;
