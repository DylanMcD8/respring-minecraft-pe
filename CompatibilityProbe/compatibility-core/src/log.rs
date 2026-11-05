/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Logging and terminal output macros.

use std::fs::File;
use std::sync::LazyLock;

/// Get a handle to the log file. This is only for use by logging macros!
///
/// All the logging macros print to stderr or (on Android) logcat, but this
/// is not convenient for users who aren't accustomed to command-line tools or
/// who don't have access to ADB, so we also write to a log file.
pub fn get_log_file() -> &'static File {
    static LOG_FILE: LazyLock<File> = LazyLock::new(|| {
        File::create(crate::paths::user_data_base_path().join("touchHLE_log.txt")).unwrap()
    });

    &LOG_FILE
}

/// Formats a log line with its trailing newline. Only for use by [echo]!
pub fn line(args: std::fmt::Arguments) -> String {
    let mut line = std::fmt::format(args);
    line.push('\n');
    line
}

#[cfg(test)]
mod tests {
    #[test]
    fn line_is_formatted_once_with_newline() {
        assert_eq!(super::line(format_args!("{}: {}", "touchHLE::mem", format_args!("{:#x}", 16))), "touchHLE::mem: 0x10\n");
        assert_eq!(super::line(format_args!("")), "\n");
    }
}

/// Prints a log message unconditionally. Use this for errors or warnings.
///
/// The message is prefixed with the module path, so it is clear where it comes
/// from.
macro_rules! log {
    ($($arg:tt)+) => {
        echo!("{}: {}", module_path!(), format_args!($($arg)+));
    }
}

/// Same as [log], but silently fails on panic instead of
/// panicking.
macro_rules! log_no_panic {
    ($($arg:tt)+) => {
        echo_no_panic!("{}: {}", module_path!(), format_args!($($arg)+));
    }
}

/// Like [log], but prints the message only if debugging is enabled for the
/// module where it is used. This can be used for verbose things only needed
/// when debugging.
macro_rules! log_dbg {
    ($($arg:tt)+) => {
        if $crate::log::ENABLED_MODULES.contains(&module_path!()) {
            log!($($arg)*);
        }
    }
}

/// Like [log], but messages only log once and cannot have formatting.
/// To be used for log messages that are known to spam the log file (like those
/// logged every frame).
macro_rules! log_once {
    ($msg:literal) => {{
        static LOG_ONCE: std::sync::Once = std::sync::Once::new();
        LOG_ONCE.call_once(|| {
            log!("{} [this log will only be shown once]", $msg);
        });
    }};
}

/// Print a message (with implicit newline). This should be used for all
/// touchHLE output that isn't coming from the app itself.
///
/// Prefer use [log] or [log_dbg] for errors and warnings during emulation.
macro_rules! echo {
    ($($arg:tt)+) => {
        {
            // Build the whole line once so each sink gets a single write.
            let line = $crate::log::line(format_args!($($arg)+));

            #[cfg(target_os = "android")]
            {
                sdl2::log::log(&line[..line.len() - 1]);
            }
            #[cfg(not(target_os = "android"))]
            let _ = std::io::Write::write_all(&mut std::io::stderr().lock(), line.as_bytes());

            use std::io::Write;
            let _ = $crate::log::get_log_file().write_all(line.as_bytes());
        }
    };
    () => {
        {
            #[cfg(target_os = "android")]
            {
                sdl2::log::log("");
            }
            #[cfg(not(target_os = "android"))]
            let _ = std::io::Write::write_all(&mut std::io::stderr().lock(), b"\n");

            use std::io::Write;
            let _ = $crate::log::get_log_file().write_all(b"\n");
        }
    }
}

/// Same as [echo], but silently fails on panic instead of
/// panicking.
macro_rules! echo_no_panic {
    ($($arg:tt)*) => {
        {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                echo!($($arg)*);
            }));
        }
    }
}

/// Put modules to enable [log_dbg] for here, e.g. "touchHLE::mem" to see when
/// memory is allocated and freed.
pub const ENABLED_MODULES: &[&str] = &[];
