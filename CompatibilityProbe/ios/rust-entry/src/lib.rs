use std::ffi::{c_char, CStr};

#[no_mangle]
pub unsafe extern "C" fn minecraft_probe_run(ipa: *const c_char, resources: *const c_char,
                                             documents: *const c_char) -> i32 {
    if ipa.is_null() || resources.is_null() || documents.is_null() { return 1; }
    let result = std::panic::catch_unwind(|| -> Result<(), String> {
        let ipa = CStr::from_ptr(ipa).to_str().map_err(|e| e.to_string())?;
        let resources = CStr::from_ptr(resources).to_str().map_err(|e| e.to_string())?;
        let documents = CStr::from_ptr(documents).to_str().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(documents).map_err(|e| e.to_string())?;
        std::env::set_var("TOUCHHLE_RESOURCE_DIR", resources);
        std::env::set_var("TOUCHHLE_USER_DATA_DIR", documents);
        std::env::set_current_dir(documents).map_err(|e| e.to_string())?;
        // Opt-in simulator profiling; normal launches do not create a trace.
        if std::env::var("MINECRAFT_PROFILE").as_deref() == Ok("1") {
            std::env::set_var("PEGGLE_C53_PC_SAMPLING", "1");
            touchHLE::diagnostics::initialize(
                &std::path::Path::new(documents).join("minecraft-profile.jsonl"), true,
            ).map_err(|e| e.to_string())?;
        }
        let panic_path = std::path::Path::new(documents).join("startup-panic.txt");
        std::panic::set_hook(Box::new(move |info| {
            let text = info.to_string();
            eprintln!("Minecraft startup: {text}");
            let _ = std::fs::write(&panic_path, text);
        }));
        // Original ARMv7 guest with verified native hot paths and native frame
        // presentation. The bundle identity keeps Peggle-only patches disabled.
        extern "C" {
            fn minecraft_native_resolution() -> bool;
            fn minecraft_expand_canvas() -> bool;
            fn minecraft_phone_height() -> u32;
            fn minecraft_uses_ipad_layout() -> bool;
            fn minecraft_background_frame_limit() -> u32;
        }
        let height = minecraft_phone_height();
        let tablet = minecraft_uses_ipad_layout();
        let native_resolution = minecraft_native_resolution();
        // The hardware compositor can present the iPad framebuffer directly.
        // The simulator's software GL driver is faster with the CA presenter.
        if cfg!(not(target_abi = "sim")) && (tablet || native_resolution)
            && std::env::var_os("MINECRAFT_NATIVE_PRESENT").is_none()
        {
            std::env::set_var("MINECRAFT_NATIVE_PRESENT", "0");
        }
        let frame_limit = minecraft_background_frame_limit();
        std::env::set_var("MINECRAFT_BACKGROUND_TERRAIN", if frame_limit == 0 { "0" } else { "1" });
        let mut args = vec![
            "Minecraft PE probe".into(),
            ipa.into(),
            "--landscape-right".into(),
            "--pause-on-background".into(),
            format!("--iphone-screen={}", if height == 480 { "3.5" } else { "4" }),
            format!("--device-family={}", if tablet { "ipad" } else { "iphone" }),
            "--no-error-popup".into(),
        ];
        if native_resolution {
            args.push("--wrapper-resolution=canvas".into());
        }
        if minecraft_expand_canvas() {
            args.push("--expand-landscape-canvas".into());
        }
        if frame_limit > 0 {
            args.push(format!("--fps-limit={frame_limit}"));
        }
        touchHLE::main(args.into_iter())
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => { eprintln!("Minecraft startup error: {e}"); 1 },
        Err(_) => 2,
    }
}

#[no_mangle]
pub extern "C" fn minecraft_refresh_sample(seconds: f64, callbacks: u32, distinct: u32) {
    touchHLE::diagnostics::event(&format!(
        "{{\"event\":\"native_refresh_window\",\"seconds\":{seconds:.5},\"callbacks\":{callbacks},\"distinct_images\":{distinct}}}"
    ));
}
