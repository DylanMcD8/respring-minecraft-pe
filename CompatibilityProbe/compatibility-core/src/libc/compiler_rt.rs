/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Compiler arithmetic helpers exported by the guest's libSystem ABI.
use crate::dyld::{export_c_func, FunctionExports};
use crate::Environment;
use crate::mem::{Mem, MutPtr};

// compiler-rt C ABI: r0/r1 are dividend/divisor, r2 is a guest pointer
// receiving the remainder, and the quotient returns in r0. This is not
// the different __aeabi_uidivmod register-pair return convention.
fn udivmod(mem: &mut Mem, a: u32, b: u32, remainder: MutPtr<u32>) -> u32 {
    assert_ne!(b, 0, "__udivmodsi4: division by zero");
    let q = a / b;
    mem.write(remainder, a % b);
    q
}
fn divmod(mem: &mut Mem, a: i32, b: i32, remainder: MutPtr<i32>) -> i32 {
    assert_ne!(b, 0, "__divmodsi4: division by zero");
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

fn __divsi3(_env: &mut Environment, a: i32, b: i32) -> i32 { a.wrapping_div(b) }
fn __udivsi3(_env: &mut Environment, a: u32, b: u32) -> u32 { a / b }
fn __modsi3(_env: &mut Environment, a: i32, b: i32) -> i32 { a.wrapping_rem(b) }
fn __umodsi3(_env: &mut Environment, a: u32, b: u32) -> u32 { a % b }
fn __divdi3(_env: &mut Environment, a: i64, b: i64) -> i64 { a.wrapping_div(b) }
fn __udivdi3(_env: &mut Environment, a: u64, b: u64) -> u64 { a / b }
fn __moddi3(_env: &mut Environment, a: i64, b: i64) -> i64 { a.wrapping_rem(b) }
fn __umoddi3(_env: &mut Environment, a: u64, b: u64) -> u64 { a % b }
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
}
