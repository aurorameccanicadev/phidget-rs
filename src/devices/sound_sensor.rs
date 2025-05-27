// phidget-rs/src/voltage_io.rs
//
// Copyright (c) 2023, Frank Pagliughi
// Copyright (c) 2025, Massimo Gismondi
//
// This file is part of the 'phidget-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use crate::{Phidget, Result, ReturnCode};
use phidget_sys::{self as ffi, PhidgetHandle, PhidgetSoundSensorHandle};
use std::{ffi::c_void, mem, ptr};

/// The function signature for the safe Rust voltage change callback.
pub type SoundSPLChangeCallback = dyn Fn(&SoundSensor, f64, f64, f64, &[f64; 10]) + Send + 'static;

/////////////////////////////////////////////////////////////////////////////

/// The function type for the safe Rust voltage input attach callback.
pub type AttachCallback = dyn Fn(&mut SoundSensor) + Send + 'static;

/// The function type for the safe Rust voltage input detach callback.
pub type DetachCallback = dyn Fn(&mut SoundSensor) + Send + 'static;

/// Phidget voltage input
pub struct SoundSensor {
    // Handle to the sound sensor input in the phidget22 library
    chan: PhidgetSoundSensorHandle,
    // Double-boxed SoundSPLChangeCallback, if registered
    cb: Option<*mut c_void>,
    // Double-boxed attach callback, if registered
    attach_cb: Option<*mut c_void>,
    // Double-boxed detach callback, if registered
    detach_cb: Option<*mut c_void>,
}

impl SoundSensor {
    /// Create a new voltage input.
    pub fn new() -> Self {
        let mut chan: PhidgetSoundSensorHandle = ptr::null_mut();
        unsafe {
            ffi::PhidgetSoundSensor_create(&mut chan);
        }
        Self::from(chan)
    }

    // Low-level, unsafe callback for device attach events
    unsafe extern "C" fn on_attach(phid: PhidgetHandle, ctx: *mut c_void) {
        if !ctx.is_null() {
            let cb: &mut Box<AttachCallback> = &mut *(ctx as *mut _);
            let mut sensor = Self::from(phid as PhidgetSoundSensorHandle);
            cb(&mut sensor);
            mem::forget(sensor);
        }
    }

    // Low-level, unsafe callback for device detach events
    unsafe extern "C" fn on_detach(phid: PhidgetHandle, ctx: *mut c_void) {
        if !ctx.is_null() {
            let cb: &mut Box<DetachCallback> = &mut *(ctx as *mut _);
            let mut sensor = Self::from(phid as PhidgetSoundSensorHandle);
            cb(&mut sensor);
            mem::forget(sensor);
        }
    }

    // Low-level, unsafe, callback for the voltage change event.
    // The context is a double-boxed pointer to the safe Rust callback.
    unsafe extern "C" fn on_spl_change(
        chan: PhidgetSoundSensorHandle,
        ctx: *mut c_void,
        db: f64,
        db_a: f64,
        db_c: f64,
        octaves: *const f64
    )
    {
        if !ctx.is_null() {
            let cb: &mut Box<SoundSPLChangeCallback> = &mut *(ctx as *mut _);
            let octaves: &[f64; 10] = std::slice::from_raw_parts(octaves, 10)
                .try_into().expect("Octaves array must be 10 elements long");
            let sensor = Self::from(chan);
            cb(&sensor, db, db_a, db_c, octaves);
            mem::forget(sensor);
        }
    }

    /// Get a reference to the underlying sensor handle
    pub fn as_channel(&self) -> &PhidgetSoundSensorHandle {
        &self.chan
    }

    /// The most recent dB SPL value that has been calculated
    pub fn db(&self) -> Result<f64> {
        let mut v: f64 = 0.0;
        ReturnCode::result(unsafe { ffi::PhidgetSoundSensor_getdB(self.chan, &mut v) })?;
        Ok(v)
    }

    /// The most recent dBA SPL value that has been calculated.
    pub fn db_a(&self) -> Result<f64>
    {
        unimplemented!()
    }

    /// The most recent dBC SPL value that has been calculated.
    pub fn db_c(&self) -> Result<f64>
    {
        unimplemented!()
    }

    /// Sets a handler to receive SPL change callbacks.
    pub fn set_on_spl_change_handler<F>(&mut self, cb: F) -> Result<()>
    where
        F: Fn(&SoundSensor, f64, f64, f64, &[f64; 10]) + Send + 'static
    {
        // 1st box is fat ptr, 2nd is regular pointer.
        let cb: Box<Box<SoundSPLChangeCallback>> = Box::new(Box::new(cb));
        let ctx = Box::into_raw(cb) as *mut c_void;
        self.cb = Some(ctx);

        ReturnCode::result(unsafe {
            
            ffi::PhidgetSoundSensor_setOnSPLChangeHandler(
                self.chan,
                Some(Self::on_spl_change),
                ctx,
            )
        })
    }

    /// Sets a handler to receive attach callbacks
    pub fn set_on_attach_handler<F>(&mut self, cb: F) -> Result<()>
    where
        F: Fn(&mut SoundSensor) + Send + 'static,
    {
        // 1st box is fat ptr, 2nd is regular pointer.
        let cb: Box<Box<AttachCallback>> = Box::new(Box::new(cb));
        let ctx = Box::into_raw(cb) as *mut c_void;

        ReturnCode::result(unsafe {
            ffi::Phidget_setOnAttachHandler(self.as_mut_handle(), Some(Self::on_attach), ctx)
        })?;
        self.attach_cb = Some(ctx);
        Ok(())
    }

    /// Sets a handler to receive detach callbacks
    pub fn set_on_detach_handler<F>(&mut self, cb: F) -> Result<()>
    where
        F: Fn(&mut SoundSensor) + Send + 'static,
    {
        // 1st box is fat ptr, 2nd is regular pointer.
        let cb: Box<Box<DetachCallback>> = Box::new(Box::new(cb));
        let ctx = Box::into_raw(cb) as *mut c_void;

        ReturnCode::result(unsafe {
            ffi::Phidget_setOnDetachHandler(self.as_mut_handle(), Some(Self::on_detach), ctx)
        })?;
        self.detach_cb = Some(ctx);
        Ok(())
    }
}

impl Phidget for SoundSensor {
    fn as_mut_handle(&mut self) -> PhidgetHandle {
        self.chan as PhidgetHandle
    }
    fn as_handle(&self) -> PhidgetHandle {
        self.chan as PhidgetHandle
    }
}

unsafe impl Send for SoundSensor {}

impl Default for SoundSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PhidgetSoundSensorHandle> for SoundSensor {
    fn from(chan: PhidgetSoundSensorHandle) -> Self {
        Self {
            chan,
            cb: None,
            attach_cb: None,
            detach_cb: None,
        }
    }
}

impl Drop for SoundSensor {
    fn drop(&mut self) {
        if let Ok(true) = self.is_open() {
            let _ = self.close();
        }
        unsafe {
            ffi::PhidgetSoundSensor_delete(&mut self.chan);
            crate::drop_cb::<SoundSPLChangeCallback>(self.cb.take());
            crate::drop_cb::<AttachCallback>(self.attach_cb.take());
            crate::drop_cb::<DetachCallback>(self.detach_cb.take());
        }
    }
}
