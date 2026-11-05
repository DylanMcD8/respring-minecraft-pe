/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Thread-specific data keys.

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::errno::{EAGAIN, EINVAL};
use crate::mem::{ConstVoidPtr, MutPtr, MutVoidPtr, Ptr};
use crate::Environment;

const PTHREAD_KEYS_MAX: i32 = 512;

#[derive(Default)]
pub struct State {
    /// The `pthread_key_t` value, with 1 subtracted, is the index into this
    /// vector. The tuple contains the thread-specific data pointers, indexed
    /// by [crate::ThreadId] (never reused; missing entries are NULL), plus the
    /// destructor pointer.
    keys: Vec<Option<(Vec<MutVoidPtr>, GuestFunction)>>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.libc_state.pthread.key
    }
}

type pthread_key_t = u32;

fn pthread_key_create(
    env: &mut Environment,
    key_ptr: MutPtr<pthread_key_t>,
    destructor: GuestFunction, // void (*destructor)(void *), may be NULL
) -> i32 {
    for (idx, slot) in State::get(env).keys.iter_mut().enumerate() {
        if slot.is_none() {
            let key: pthread_key_t = (idx + 1).try_into().unwrap();
            *slot = Some((Vec::new(), destructor));
            env.mem.write(key_ptr, key);
            return 0; // success, reused old slot
        }
    }
    if State::get(env).keys.len() < PTHREAD_KEYS_MAX as usize {
        let idx = State::get(env).keys.len();
        let key: pthread_key_t = (idx + 1).try_into().unwrap(); // can unwrap, PTHREAD_KEYS_MAX is small enough
        State::get(env)
            .keys
            .push(Some((Vec::new(), destructor)));
        env.mem.write(key_ptr, key);
        return 0; // success, created new slot
    }
    EAGAIN // failure, no slots left
}

fn pthread_getspecific(env: &mut Environment, key: pthread_key_t) -> MutVoidPtr {
    // Use of invalid key is undefined, panicking is fine.
    let idx: usize = key.checked_sub(1).unwrap().try_into().unwrap();
    let current_thread = env.current_thread;
    match State::get(env).keys.get(idx) {
        Some(Some((values, _))) => values.get(current_thread).copied().unwrap_or(Ptr::null()),
        _ => Ptr::null(), // failure, NULL for a deleted key, UB for an invalid key
    }
}

fn pthread_setspecific(env: &mut Environment, key: pthread_key_t, value: ConstVoidPtr) -> i32 {
    // TODO: return error instead of panicking if key is invalid?
    let idx: usize = key.checked_sub(1).unwrap().try_into().unwrap();
    let current_thread = env.current_thread;
    match State::get(env).keys.get_mut(idx) {
        Some(Some((values, _))) => {
            if values.len() <= current_thread {
                values.resize(current_thread + 1, Ptr::null());
            }
            values[current_thread] = value.cast_mut();
            0 // success
        }
        _ => EINVAL, // failure, unused key or out of bounds
    }
}

fn pthread_key_delete(env: &mut Environment, key: pthread_key_t) -> i32 {
    // Use of invalid key is undefined, panicking is fine.
    let idx: usize = key.checked_sub(1).unwrap().try_into().unwrap();
    match State::get(env).keys.get_mut(idx) {
        Some(slot @ Some(_)) => {
            *slot = None;
            0 // success
        }
        _ => EINVAL, // failure, unused key or out of bounds
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(pthread_key_create(_, _)),
    export_c_func!(pthread_getspecific(_)),
    export_c_func!(pthread_setspecific(_, _)),
    export_c_func!(pthread_key_delete(_)),
];

#[cfg(test)]
pub(crate) fn minecraft_probe_thread_specific(env: &mut Environment) {
    let key_ptr = env.mem.alloc(8).cast::<pthread_key_t>();
    let destructor = GuestFunction::from_addr_with_thumb_bit(0x1235);
    assert_eq!(pthread_key_create(env, key_ptr, destructor), 0);
    assert_eq!(pthread_key_create(env, key_ptr + 1, destructor), 0);
    let (a, b) = (env.mem.read(key_ptr), env.mem.read(key_ptr + 1));
    assert_ne!(a, b);
    let value = ConstVoidPtr::from_bits;
    let thread = env.current_thread;
    // Thread ids past the end of either Vec read NULL and grow it on write.
    let (other, unused) = (thread + 5, thread + 3);

    assert!(pthread_getspecific(env, a).is_null());
    assert_eq!(pthread_setspecific(env, a, value(0x1000)), 0);
    env.current_thread = other;
    assert!(pthread_getspecific(env, a).is_null());
    assert_eq!(pthread_setspecific(env, a, value(0x5000)), 0);
    assert_eq!(pthread_setspecific(env, b, value(0x5b00)), 0);
    env.current_thread = unused;
    assert!(pthread_getspecific(env, a).is_null());
    assert!(pthread_getspecific(env, b).is_null());
    env.current_thread = thread;
    assert_eq!(pthread_getspecific(env, a).to_bits(), 0x1000);
    assert!(pthread_getspecific(env, b).is_null());
    assert_eq!(pthread_setspecific(env, a, Ptr::null()), 0);
    assert!(pthread_getspecific(env, a).is_null());

    // A deleted key reads NULL, rejects writes, and its slot is reused
    // without the old values.
    assert_eq!(pthread_key_delete(env, a), 0);
    assert!(pthread_getspecific(env, a).is_null());
    assert_eq!(pthread_setspecific(env, a, value(0x1)), EINVAL);
    assert_eq!(pthread_key_delete(env, a), EINVAL);
    assert_eq!(pthread_key_create(env, key_ptr, destructor), 0);
    assert_eq!(env.mem.read(key_ptr), a);
    env.current_thread = other;
    assert!(pthread_getspecific(env, a).is_null());
    assert_eq!(pthread_getspecific(env, b).to_bits(), 0x5b00);
    env.current_thread = thread;
    assert_eq!((pthread_key_delete(env, a), pthread_key_delete(env, b)), (0, 0));
}
