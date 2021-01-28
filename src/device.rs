//! Types and traits related to the root-level device and system-wide lifecycle events.

use crate::bus::{EventBus, EventConsumer};
use crate::supervisor::Supervisor;
use core::cell::UnsafeCell;

/// System-wide lifecycle events.
///
/// See also `NotificationHandler<...>`.  Each actor within the system is
/// required to implement `NotificationHandler<Lifecycle>` but may opt to
/// ignore any or all of the events.
#[derive(Copy, Clone, Debug)]
pub enum Lifecycle {
    /// Called after mounting but prior to starting the async executor.
    Initialize,
    /// Called after `Initialize` but prior to starting the async executor.
    Start,
    /// Not currently used.
    Stop,
    /// Not currently used.
    Sleep,
    /// Not currently used.
    Hibernate,
}

/// Trait which must be implemented by all top-level devices which
/// subsequently contain `ActorContext` or `InterruptContext` or other
/// packages.
pub trait Device {

    /// Called when the device is mounted into the system.
    ///
    /// The device *must* propagate the call through to all children `ActorContext`
    /// and `InterruptContext`, either directly or indirectly, in order for them
    /// to be mounted into the system.
    fn mount(&'static mut self, bus: &EventBus<Self>, supervisor: &mut Supervisor)
    where
        Self: Sized;
}

#[doc(hidden)]
pub struct DeviceContext<D: Device> {
    device: UnsafeCell<D>,
    supervisor: UnsafeCell<Supervisor>,
}

impl<D: Device> DeviceContext<D> {
    pub fn new(device: D) -> Self {
        Self {
            device: UnsafeCell::new(device),
            supervisor: UnsafeCell::new(Supervisor::new()),
        }
    }

    pub fn mount(&'static self) -> ! {
        let bus = EventBus::new(self);
        unsafe {
            (&mut *self.device.get()).mount(&bus, &mut *self.supervisor.get());
            (&*self.supervisor.get()).run_forever()
        }
    }

    pub fn on_interrupt(&'static self, irqn: i16) {
        unsafe {
            (&*self.supervisor.get()).on_interrupt(irqn);
        }
    }

    pub fn on_event<E>(&'static self, event: E)
    where
        D: EventConsumer<E>,
    {
        unsafe {
            (&mut *self.device.get()).on_event(event);
        }
    }
}
