/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `ifaddrs.h` (interface addresses)

use crate::dyld::FunctionExports;
use crate::export_c_func;
use crate::libc::errno::{set_errno, EINVAL, ENOTSUP};
use crate::mem::MutPtr;
use crate::Environment;

// TODO: struct definition
#[allow(non_camel_case_types)]
struct ifaddrs {}

fn getifaddrs(env: &mut Environment, ifap: MutPtr<MutPtr<ifaddrs>>) -> i32 {
    if ifap.is_null() { set_errno(env,EINVAL); return -1; }
    env.mem.write(ifap,MutPtr::null());
    // Interface enumeration is not implemented; report a real failure.
    set_errno(env, ENOTSUP);
    -1
}

fn freeifaddrs(_env: &mut Environment, ifa: MutPtr<ifaddrs>) {
    // No list is allocated by getifaddrs yet. Freeing its null result is safe.
    assert!(ifa.is_null(), "freeifaddrs: unexpected non-null interface list");
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(getifaddrs(_)), export_c_func!(freeifaddrs(_))];
