/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Compiler arithmetic helpers exported by the guest's libSystem ABI.
use crate::dyld::{export_c_func, FunctionExports};
use crate::Environment;
use crate::mem::{Mem, MutPtr};

// A zero divisor doesn't trap on device: Darwin's ARM compiler-rt is not
// EABI, so there is no __aeabi_idiv0 call, and the hardware-divide paths
// (armv7s slice, which 32-bit code on A6 and later devices uses) return 0
// without storing a remainder. Match that rather than aborting the emulator.
macro_rules! div0 {
    ($a:ident, $b:ident, $op:ident) => { if $b == 0 { 0 } else { $a.$op($b) } };
}

// compiler-rt C ABI: r0/r1 are dividend/divisor, r2 is a guest pointer
// receiving the remainder, and the quotient returns in r0. This is not
// the different __aeabi_uidivmod register-pair return convention.
fn udivmod(mem: &mut Mem, a: u32, b: u32, remainder: MutPtr<u32>) -> u32 {
    if b == 0 { return 0; }
    let q = a / b;
    mem.write(remainder, a % b);
    q
}
fn divmod(mem: &mut Mem, a: i32, b: i32, remainder: MutPtr<i32>) -> i32 {
    if b == 0 { return 0; }
    let q = a.wrapping_div(b);
    mem.write(remainder, a.wrapping_rem(b));
    q
}
fn __udivmodsi4(env: &mut Environment, a: u32, b: u32, remainder: MutPtr<u32>) -> u32 {
    udivmod(&mut env.mem, a, b, remainder)
}
fn __divmodsi4(env: &mut Environment, a: i32, b: i32, remainder: MutPtr<i32>) -> i32 {
    divmod(&mut env.mem, a, b, remainder)
}

fn __divsi3(_env: &mut Environment, a: i32, b: i32) -> i32 { div0!(a, b, wrapping_div) }
fn __udivsi3(_env: &mut Environment, a: u32, b: u32) -> u32 { div0!(a, b, wrapping_div) }
fn __modsi3(_env: &mut Environment, a: i32, b: i32) -> i32 { div0!(a, b, wrapping_rem) }
fn __umodsi3(_env: &mut Environment, a: u32, b: u32) -> u32 { div0!(a, b, wrapping_rem) }
fn __divdi3(_env: &mut Environment, a: i64, b: i64) -> i64 { div0!(a, b, wrapping_div) }
fn __udivdi3(_env: &mut Environment, a: u64, b: u64) -> u64 { div0!(a, b, wrapping_div) }
fn __moddi3(_env: &mut Environment, a: i64, b: i64) -> i64 { div0!(a, b, wrapping_rem) }
fn __umoddi3(_env: &mut Environment, a: u64, b: u64) -> u64 { div0!(a, b, wrapping_rem) }
fn __fixdfdi(_env: &mut Environment, a: f64) -> i64 { a as i64 }
fn __fixsfdi(_env: &mut Environment, a: f32) -> i64 { a as i64 }
fn __fixunssfdi(_env: &mut Environment, a: f32) -> u64 { a as u64 }
fn __floatdidf(_env: &mut Environment, a: i64) -> f64 { a as f64 }
fn __floatdisf(_env: &mut Environment, a: i64) -> f32 { a as f32 }
fn __floatundidf(_env: &mut Environment, a: u64) -> f64 { a as f64 }

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(__udivmodsi4(_, _, _)), export_c_func!(__divmodsi4(_, _, _)),
    export_c_func!(__divsi3(_, _)), export_c_func!(__udivsi3(_, _)),
    export_c_func!(__modsi3(_, _)), export_c_func!(__umodsi3(_, _)),
    export_c_func!(__divdi3(_, _)), export_c_func!(__udivdi3(_, _)),
    export_c_func!(__moddi3(_, _)),
    export_c_func!(__umoddi3(_, _)), export_c_func!(__fixdfdi(_)),
    export_c_func!(__fixsfdi(_)), export_c_func!(__fixunssfdi(_)),
    export_c_func!(__floatdidf(_)), export_c_func!(__floatdisf(_)),
    export_c_func!(__floatundidf(_)),
];

#[cfg(test)]
pub(crate) fn minecraft_probe_signed_64bit_remainder_guest_abi(env: &mut Environment) {
    use crate::abi::CallFromHost;
    let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "___moddi3").unwrap();
    for (a,b,expected) in [
        (0i64,7i64,0i64), (7,3,1), (-7,3,-1), (7,-3,1), (-7,-3,-1),
        (i64::MIN,3,-2), (i64::MIN,-1,0), (i64::MAX,i64::MIN,i64::MAX),
        (i64::MIN,i64::MIN,0), ((1i64<<40)+5,(1i64<<32)+3,4294966536),
        (-((1i64<<40)+5),(1i64<<32)+3,-4294966536),
        ((1i64<<40)+5,-((1i64<<32)+3),4294966536),
    ] {
        let result: i64 = f.call_from_host(env,(a,b));
        assert_eq!(result,expected,"{a} remainder {b}");
    }
}

#[cfg(test)]
pub(crate) fn minecraft_probe_zero_divisor_guest_abi(env: &mut Environment) {
    use crate::abi::CallFromHost;
    let out = env.mem.alloc_and_write(0xdeadu32);
    for name in ["___udivmodsi4","___divmodsi4"] {
        let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, name).unwrap();
        for a in [0u32, 5, 0x8000_0000, u32::MAX] {
            let r: u32 = f.call_from_host(env,(a,0u32,out));
            assert_eq!((r,env.mem.read(out)),(0,0xdead),"{name}({a:#x}, 0)");
        }
        let r: u32 = f.call_from_host(env,(7u32,2u32,out));
        assert_eq!((r,env.mem.read(out)),(3,1),"{name}(7, 2)");
        env.mem.write(out, 0xdead);
    }
    for name in ["___divsi3","___udivsi3","___modsi3","___umodsi3"] {
        let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, name).unwrap();
        for a in [0u32, 5, 0x8000_0000, u32::MAX] {
            let r: u32 = f.call_from_host(env,(a,0u32));
            assert_eq!(r,0,"{name}({a:#x}, 0)");
        }
        let r: u32 = f.call_from_host(env,(7u32,2u32));
        assert_eq!(r,if name.contains("mod") {1} else {3},"{name}(7, 2)");
    }
    for name in ["___divdi3","___udivdi3","___moddi3","___umoddi3"] {
        let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, name).unwrap();
        for a in [0u64, 5, 1<<40, u64::MAX] {
            let r: u64 = f.call_from_host(env,(a,0u64));
            assert_eq!(r,0,"{name}({a:#x}, 0)");
        }
    }
    // Signed overflow still wraps rather than trapping.
    let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "___divsi3").unwrap();
    let r: i32 = f.call_from_host(env,(i32::MIN,-1i32));
    assert_eq!(r,i32::MIN);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minecraft_probe_divmod_guest_memory_and_boundaries() {
        let mut mem = Mem::new();
        let storage = mem.alloc(12).cast::<u32>();
        mem.write(storage, 0x12345678);
        mem.write(storage + 2, 0xabcdef01);
        let out = storage + 1;
        for (a,b,q,r) in [(292,100,2,92), (0,7,0,0), (7,9,0,7),
                         (u32::MAX,1,u32::MAX,0), (u32::MAX,2,2147483647,1),
                         (u32::MAX,u32::MAX,1,0)] {
            assert_eq!(udivmod(&mut mem,a,b,out),q);
            assert_eq!(mem.read(out),r);
        }
        for (a,b,q,r) in [(-7,3,-2,-1), (7,-3,-2,1), (-7,-3,2,-1),
                         (i32::MIN,-1,i32::MIN,0), (i32::MIN,3,-715827882,-2),
                         (i32::MAX,2,1073741823,1)] {
            assert_eq!(divmod(&mut mem,a,b,out.cast()),q);
            assert_eq!(mem.read(out.cast::<i32>()),r);
        }
        assert_eq!(mem.read(storage),0x12345678);
        assert_eq!(mem.read(storage+2),0xabcdef01);
    }

    #[test]
    fn zero_divisor_returns_zero_like_device() {
        let mut mem = Mem::new();
        let out = mem.alloc(4).cast::<u32>();
        for a in [0u32, 1, 7, u32::MAX] {
            mem.write(out, 0xdead);
            assert_eq!(udivmod(&mut mem,a,0,out),0);
            assert_eq!(mem.read(out),0xdead);
            assert_eq!(divmod(&mut mem,a as i32,0,out.cast()),0);
            assert_eq!(mem.read(out),0xdead);
        }
    }
}
