/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `wchar.h`

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, GuestUSize, MutPtr};
use crate::Environment;

use super::generic_char::GenericChar;

#[allow(non_camel_case_types)]
pub type wchar_t = i32; // not sure if this signedness is correct

#[allow(non_camel_case_types)]
type wint_t = i32;

const WEOF: wint_t = -1;

// The initial libc++ locale is the stateless C locale. Do not silently
// treat an explicit (as yet unsupported) guest locale handle as C.
fn mbtowc_l(env: &mut Environment, out: MutPtr<wchar_t>, input: ConstPtr<u8>,
            count: GuestUSize, locale: u32) -> i32 {
    assert_eq!(locale, 0, "mbtowc_l: explicit guest locale is unsupported");
    if input.is_null() { return 0; }
    if count == 0 {
        super::errno::set_errno(env, 92); // Darwin EILSEQ
        return -1;
    }
    let byte = env.mem.read(input);
    if !byte.is_ascii() {
        super::errno::set_errno(env, 92);
        return -1;
    }
    if !out.is_null() { env.mem.write(out, byte as wchar_t); }
    i32::from(byte != 0)
}

fn btowc(_env: &mut Environment, c: i32) -> wint_t {
    let c = c as u8;
    // Assuming ASCII locale, like in ctype.rs.
    if c.is_ascii() {
        c as wint_t
    } else {
        WEOF
    }
}

fn wctob(_env: &mut Environment, c: wint_t) -> i32 {
    // Assuming ASCII locale, like in ctype.rs.
    if u32::try_from(c)
        .ok()
        .and_then(char::from_u32)
        .is_some_and(|c| c.is_ascii())
    {
        c
    } else {
        WEOF
    }
}

// Functions shared with string.rs

fn wmemset(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    ch: wchar_t,
    count: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::memset(env, dest, ch, count, GuestUSize::MAX)
}
fn wmemcpy(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::memcpy(env, dest, src, size, GuestUSize::MAX)
}
fn __wmemcpy_chk(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
    dest_size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::memcpy(env, dest, src, size, dest_size)
}
fn wmemmove(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::memmove(env, dest, src, size, GuestUSize::MAX)
}
fn __wmemmove_chk(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
    dest_size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::memmove(env, dest, src, size, dest_size)
}
fn wmemchr(
    env: &mut Environment,
    string: ConstPtr<wchar_t>,
    c: wchar_t,
    size: GuestUSize,
) -> ConstPtr<wchar_t> {
    GenericChar::<wchar_t>::memchr(env, string, c, size)
}
fn wmemcmp(
    env: &mut Environment,
    a: ConstPtr<wchar_t>,
    b: ConstPtr<wchar_t>,
    size: GuestUSize,
) -> i32 {
    GenericChar::<wchar_t>::memcmp(env, a, b, size)
}
fn wcslen(env: &mut Environment, s: ConstPtr<wchar_t>) -> GuestUSize {
    GenericChar::<wchar_t>::strlen(env, s)
}
fn wcscpy(env: &mut Environment, dest: MutPtr<wchar_t>, src: ConstPtr<wchar_t>) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strcpy(env, dest, src, GuestUSize::MAX)
}
fn wcscat(env: &mut Environment, dest: MutPtr<wchar_t>, src: ConstPtr<wchar_t>) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strcat(env, dest, src, GuestUSize::MAX)
}
fn wcsspn(env: &mut Environment, s: ConstPtr<wchar_t>, charset: ConstPtr<wchar_t>) -> GuestUSize {
    GenericChar::<wchar_t>::strspn(env, s, charset)
}
fn wcscspn(
    env: &mut Environment,
    str: ConstPtr<wchar_t>,
    charset: ConstPtr<wchar_t>,
) -> GuestUSize {
    GenericChar::<wchar_t>::strcspn(env, str, charset)
}
fn wcsncpy(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strncpy(env, dest, src, size, GuestUSize::MAX)
}
fn __wcsncpy_chk(
    env: &mut Environment,
    dest: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
    dest_size: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strncpy(env, dest, src, size, dest_size)
}
fn wcsdup(env: &mut Environment, src: ConstPtr<wchar_t>) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strdup(env, src)
}
fn wcscmp(env: &mut Environment, a: ConstPtr<wchar_t>, b: ConstPtr<wchar_t>) -> i32 {
    GenericChar::<wchar_t>::strcmp(env, a, b)
}
fn wcsncmp(
    env: &mut Environment,
    a: ConstPtr<wchar_t>,
    b: ConstPtr<wchar_t>,
    n: GuestUSize,
) -> i32 {
    GenericChar::<wchar_t>::strncmp(env, a, b, n)
}
fn wcsncat(
    env: &mut Environment,
    s1: MutPtr<wchar_t>,
    s2: ConstPtr<wchar_t>,
    n: GuestUSize,
) -> MutPtr<wchar_t> {
    GenericChar::<wchar_t>::strncat(env, s1, s2, n)
}
fn wcsstr(
    env: &mut Environment,
    wcsing: ConstPtr<wchar_t>,
    subwcsing: ConstPtr<wchar_t>,
) -> ConstPtr<wchar_t> {
    GenericChar::<wchar_t>::strstr(env, wcsing, subwcsing)
}
fn wcschr(env: &mut Environment, wcsing: ConstPtr<wchar_t>, wchar: wchar_t) -> ConstPtr<wchar_t> {
    GenericChar::<wchar_t>::strchr(env, wcsing, wchar)
}
fn wcsrchr(env: &mut Environment, wcsing: ConstPtr<wchar_t>, wchar: wchar_t) -> ConstPtr<wchar_t> {
    GenericChar::<wchar_t>::strrchr(env, wcsing, wchar)
}
fn wcspbrk(
    env: &mut Environment,
    s: ConstPtr<wchar_t>,
    charset: ConstPtr<wchar_t>,
) -> ConstPtr<wchar_t> {
    GenericChar::<wchar_t>::strpbrk(env, s, charset)
}
fn wcslcpy(
    env: &mut Environment,
    dst: MutPtr<wchar_t>,
    src: ConstPtr<wchar_t>,
    size: GuestUSize,
) -> GuestUSize {
    GenericChar::<wchar_t>::strlcpy(env, dst, src, size)
}

fn wcstok(env: &mut Environment, string: MutPtr<wchar_t>, delimiters: ConstPtr<wchar_t>, context: MutPtr<MutPtr<wchar_t>>) -> MutPtr<wchar_t> {
    let current=if string.is_null() { env.mem.read(context) } else { string };
    if current.is_null() { return MutPtr::null(); }
    let length=GenericChar::<wchar_t>::strlen(env,current.cast_const());
    let delimiter_length=GenericChar::<wchar_t>::strlen(env,delimiters);
    let text: Vec<wchar_t>=(0..=length).map(|i|env.mem.read(current+i)).collect();
    let separators: Vec<wchar_t>=(0..=delimiter_length).map(|i|env.mem.read(delimiters+i)).collect();
    let Some((start,end))=crate::wide_token::next_token_range(&text,&separators) else {
        env.mem.write(context,MutPtr::null());return MutPtr::null();
    };
    let next=if text[end]!=0 { env.mem.write(current+end as u32,0);current+end as u32+1 } else { MutPtr::null() };
    env.mem.write(context,next);
    current+start as u32
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(mbtowc_l(_, _, _, _)),
    export_c_func!(wcstok(_, _, _)),
    export_c_func!(btowc(_)),
    export_c_func!(wctob(_)),
    // Functions shared with string.rs
    export_c_func!(wmemset(_, _, _)),
    export_c_func!(wmemcpy(_, _, _)),
    export_c_func!(__wmemcpy_chk(_, _, _, _)),
    export_c_func!(wmemmove(_, _, _)),
    export_c_func!(__wmemmove_chk(_, _, _, _)),
    export_c_func!(wmemchr(_, _, _)),
    export_c_func!(wmemcmp(_, _, _)),
    export_c_func!(wcslen(_)),
    export_c_func!(wcscpy(_, _)),
    export_c_func!(wcscat(_, _)),
    export_c_func!(wcsspn(_, _)),
    export_c_func!(wcscspn(_, _)),
    export_c_func!(wcsncpy(_, _, _)),
    export_c_func!(__wcsncpy_chk(_, _, _, _)),
    export_c_func!(wcsdup(_)),
    export_c_func!(wcscmp(_, _)),
    export_c_func!(wcsncmp(_, _, _)),
    export_c_func!(wcsncat(_, _, _)),
    export_c_func!(wcsstr(_, _)),
    export_c_func!(wcschr(_, _)),
    export_c_func!(wcsrchr(_, _)),
    export_c_func!(wcspbrk(_, _)),
    export_c_func!(wcslcpy(_, _, _)),
];

#[cfg(test)]
mod minecraft_probe_tests {
    use super::*;
    use crate::abi::CallFromHost;
    #[test]
    fn minecraft_probe_locale_and_divmod_guest_abi() {
        let options = crate::options::Options { headless: true, popup_errors: false, ..Default::default() };
        let icon = crate::image::Image::from_pixel_vec(vec![0; 4], (1, 1));
        Environment::new_without_app(options, icon).unwrap().run_app_picker(|env| {
            crate::libc::pthread::cond::minecraft_probe_fixture(env);
            crate::libc::compiler_rt::minecraft_probe_signed_64bit_remainder_guest_abi(env);
            crate::libc::compiler_rt::minecraft_probe_zero_divisor_guest_abi(env);
            crate::libc::math::minecraft_probe_nearbyintf_guest_abi(env);
            crate::libc::stdlib::minecraft_probe_free_resets_errno(env);
            crate::libc::posix_io::minecraft_probe_close_and_sync(env);
            crate::libc::pthread::key::minecraft_probe_thread_specific(env);
            crate::libc::stdio::minecraft_probe_fread_short_eof(env);
            let sendto = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "_sendto").unwrap();
            let error = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "___error").unwrap();
            // Invalid descriptors must fail before inspecting packet pointers.
            for (fd, expected) in [(-1,9),(0,38),(1,38),(2,38),(9999,9)] {
                let result: i32 = sendto.call_from_host(env,(fd,0u32,25u32,0i32,0u32,16u32));
                assert_eq!(result,-1);
                let errno: ConstPtr<i32> = error.call_from_host(env,());
                assert_eq!(env.mem.read(errno),expected);
            }
            let address_fn = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "_inet_ntoa").unwrap();
            let mut previous = ConstPtr::<u8>::null();
            for (address,expected) in [(0u32,"0.0.0.0"),(0x0100007f,"127.0.0.1"),(u32::MAX,"255.255.255.255"),(0,"0.0.0.0")] {
                let ptr: ConstPtr<u8> = address_fn.call_from_host(env,(address,));
                assert_eq!(env.mem.cstr_at_utf8(ptr).unwrap(),expected);
                if !previous.is_null() { assert_eq!(ptr,previous); }
                previous=ptr;
            }
            let format_fn = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "_sprintf_l").unwrap();
            let buffer = env.mem.alloc(64).cast::<u8>();
            let format = env.mem.alloc_and_write_cstr(b"%d %.2f %s");
            let word = env.mem.alloc_and_write_cstr(b"ok");
            let count: i32 = format_fn.call_from_host(env, (buffer, 0u32, format.cast_const(), 17i32, 1.5f64, word.cast_const()));
            assert_eq!(count, 10);
            assert_eq!(env.mem.cstr_at(buffer.cast_const()), b"17 1.50 ok");
            let bounded_fn = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "_snprintf_l").unwrap();
            env.mem.bytes_at_mut(buffer,64).fill(0x55);
            let count: i32 = bounded_fn.call_from_host(env, (buffer, 4u32, 0u32, format.cast_const(), 17i32, 1.5f64, word.cast_const()));
            assert_eq!(count,10);
            assert_eq!(env.mem.cstr_at(buffer.cast_const()),b"17 ");
            assert_eq!(env.mem.read(buffer+4),0x55);
            let f = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "_mbtowc_l").unwrap();
            let out = env.mem.alloc_and_write(0x12345678i32);
            let reset: i32 = f.call_from_host(env, (out, ConstPtr::<u8>::null(), 6u32, 0u32));
            assert_eq!(reset, 0);
            assert_eq!(env.mem.read(out), 0x12345678);
            let input = env.mem.alloc_and_write_cstr(b"A");
            let converted: i32 = f.call_from_host(env, (out, input.cast_const(), 1u32, 0u32));
            assert_eq!(converted, 1);
            assert_eq!(env.mem.read(out), 65);
            let nul: i32 = f.call_from_host(env, (out, (input+1).cast_const(), 1u32, 0u32));
            assert_eq!(nul, 0);
            assert_eq!(env.mem.read(out), 0);
            env.mem.write(input, 0xffu8);
            let invalid: i32 = f.call_from_host(env, (out, input.cast_const(), 1u32, 0u32));
            assert_eq!(invalid, -1);
            assert_eq!(env.mem.read(out), 0);
            let zero_length: i32 = f.call_from_host(env, (out, input.cast_const(), 0u32, 0u32));
            assert_eq!(zero_length, -1);
            let div = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "___udivmodsi4").unwrap();
            let quotient: u32 = div.call_from_host(env, (292u32, 100u32, out.cast::<u32>()));
            assert_eq!(quotient, 2);
            assert_eq!(env.mem.read(out), 92);
            let div = env.dyld.create_proc_address(&mut env.mem, &mut env.cpu, "___divmodsi4").unwrap();
            let quotient: i32 = div.call_from_host(env, (-7i32, 3i32, out));
            assert_eq!(quotient, -2);
            assert_eq!(env.mem.read(out), -1);
        });
    }
}
