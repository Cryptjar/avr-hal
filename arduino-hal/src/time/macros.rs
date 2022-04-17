// Stuff to be used in the macros

pub use crate::clock::Clock as CpuClock;
pub use avr_device::interrupt::free as interrupt_free;
pub use avr_device::interrupt::CriticalSection;
pub use avr_device::interrupt::Mutex;
pub use core::cell::Cell;
pub use core::cell::RefCell;

/// Creates a timepiece for the [timer peripheral](crate::time::timers) and
/// the given configuration
///
/// # Example
///
/// ```
/// use arduino_hal::impl_timepiece;
/// impl_timepiece! {
///     pub timepiece MyFooTimer {
///         hal: arduino_hal::hal::Atmega,
///         peripheral: Timer0,
///         cpu_clock: arduino_hal::DefaultClock,
///         millis: u32,
///         micros: u32,
///         resolution: crate::time::Resolution::_1_MS,
///     }
/// }
/// ```
///
#[macro_export]
macro_rules! impl_timepiece {
    (
        $(#[$meta:meta])*
        $vis:vis timepiece $Name: ident {
            peripheral: $TC:ident,
            cpu_clock: $CLOCK:ty,
            millis: $MILLIS:ty,
            micros: $MICROS:ty,
            resolution: $resolution:expr,
        }
    ) => {
        // The timer interrupt service routine
        $crate::hal::attach_timing_circuit_interrupt!{$TC; {
            // Increment the "millis" counter
            $crate::time::macros::interrupt_free(|cs| {
                $crate::time::update_timer::<$crate::hal::HAL, $Name>(cs)
            })
        }}

        $(#[$meta])*
        $vis struct $Name {
            pub peripheral: $crate::hal::time::$TC,
        }

        unsafe impl $crate::time::Timepiece<$crate::hal::HAL> for $Name {
            // SAFETY: We registered the interrupt for $TC above
            type Circuit = $crate::hal::time::$TC;
            type CpuClock = $CLOCK;
            type Millis = $MILLIS;
            type Micros = $MICROS;

            const RESOLUTION: Self::Millis = {
                // Ensure that `$resolution` is a Resolution
                let res: $crate::time::Resolution = $resolution;
                let millis = $resolution.as_ms();

                // TODO: use `Into`, but it is not const yet
                millis as Self::Millis
            };

            const TIMER_PARAMS: ($crate::time::Prescaler, <Self::Circuit as $crate::time::TimingCircuitOps<$crate::hal::HAL>>::Counter) = {
                // Ensure that `$resolution` is a Resolution
                let res: $crate::time::Resolution = $resolution;
                let (prescaler, cnt_top) = res.params_for_frq(
                    <Self::CpuClock as $crate::time::macros::CpuClock>::FREQ,
                    <Self::Circuit as $crate::time::TimingCircuitOps<$crate::hal::HAL>>::Counter::MAX as u32 /* TODO: use `Into` */
                ).unwrap();

                (prescaler, cnt_top as _)
            };

            fn access_millis(cs: &$crate::time::macros::CriticalSection) -> & $crate::time::macros::Cell<Self::Millis> {
                // Counts proper milliseconds
                static MILLIS_COUNTER: $crate::time::macros::Mutex<$crate::time::macros::Cell<$MILLIS>> = $crate::time::macros::Mutex::new($crate::time::macros::Cell::new(0));

                MILLIS_COUNTER.borrow(cs)
            }

            fn access_peripheral(&self) -> &Self::Circuit {
                &self.peripheral
            }
        }
    };
}

#[cfg(test)]
pub mod test {
    // TODO: needs
    // #![feature(abi_avr_interrupt)]
    // #![feature(const_option)]

    impl_timepiece! {
        pub timepiece MyFooTimer {
            peripheral: Timer0,
            cpu_clock: crate::DefaultClock,
            millis: u32,
            micros: u32,
            resolution: crate::time::Resolution::MS_1,
        }
    }
}