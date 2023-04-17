// phidget-rs/src/temperature_sensor.rs
//
// Copyright (c) 2023, Frank Pagliughi
//
// This file is part of the 'phidget-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use crate::{Phidget, Result};
use phidget_sys::{
    self as ffi, PhidgetHandle, PhidgetTemperatureSensorHandle as TemperatureSensorHandle,
};
use std::{mem, os::raw::c_void, ptr};

pub type TemperatureCallback = dyn Fn(&TemperatureSensor, f64) + Send + 'static;

/// Phidget temperature sensor
pub struct TemperatureSensor {
    // Handle to the sensor for the phidget22 library
    chan: TemperatureSensorHandle,
    // Double-boxed TemperatureCallback, if registered
    cb: Option<*mut c_void>,
}

impl TemperatureSensor {
    /// Create a new temperature sensor.
    pub fn new() -> Self {
        let mut chan: TemperatureSensorHandle = ptr::null_mut();
        unsafe {
            ffi::PhidgetTemperatureSensor_create(&mut chan);
        }
        Self { chan, cb: None }
    }

    // Low-level, unsafe, callback for temperature change events.
    // The context is a double-boxed pointer the the safe Rust callback.
    unsafe extern "C" fn on_temperature_change(
        chan: TemperatureSensorHandle,
        ctx: *mut c_void,
        temperature: f64,
    ) {
        if !ctx.is_null() {
            let cb: &mut Box<TemperatureCallback> = &mut *(ctx as *mut _);
            let sensor = Self { chan, cb: None };
            cb(&sensor, temperature);
            mem::forget(sensor);
        }
    }

    // Drop/delete the humidity change callback
    // This must not be done if the callback is running
    unsafe fn drop_callback(&mut self) {
        if let Some(ctx) = self.cb.take() {
            let _: Box<Box<TemperatureCallback>> = unsafe { Box::from_raw(ctx as *mut _) };
        }
    }

    /// Get a reference to the underlying sensor handle
    pub fn as_channel(&self) -> &TemperatureSensorHandle {
        &self.chan
    }

    /// Read the current temperature
    pub fn temperature(&self) -> Result<f64> {
        let mut temperature = 0.0;
        unsafe {
            crate::check_ret(ffi::PhidgetTemperatureSensor_getTemperature(
                self.chan,
                &mut temperature,
            ))?;
        }
        Ok(temperature)
    }

    /// Set a handler to receive temperature change callbacks.
    pub fn set_on_temperature_change_handler<F>(&mut self, cb: F) -> Result<()>
    where
        F: Fn(&TemperatureSensor, f64) + Send + 'static,
    {
        // 1st box is fat ptr, 2nd is regular pointer.
        let cb: Box<Box<TemperatureCallback>> = Box::new(Box::new(cb));
        let ctx = Box::into_raw(cb) as *mut c_void;
        self.cb = Some(ctx);

        unsafe {
            crate::check_ret(ffi::PhidgetTemperatureSensor_setOnTemperatureChangeHandler(
                self.chan,
                Some(Self::on_temperature_change),
                ctx,
            ))
        }
    }

    /// Removes the temperature change callback
    pub fn remove_on_temperature_change_handler(&mut self) -> Result<()> {
        unsafe {
            let ret =
                crate::check_ret(ffi::PhidgetTemperatureSensor_setOnTemperatureChangeHandler(
                    self.chan,
                    None,
                    ptr::null_mut(),
                ));
            self.drop_callback();
            ret
        }
    }
}

impl Phidget for TemperatureSensor {
    fn as_handle(&mut self) -> PhidgetHandle {
        self.chan as PhidgetHandle
    }
}

impl Default for TemperatureSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TemperatureSensor {
    fn drop(&mut self) {
        unsafe {
            ffi::PhidgetTemperatureSensor_delete(&mut self.chan);
            self.drop_callback();
        }
    }
}