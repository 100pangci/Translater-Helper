use std::{env, path::Path};

/// Avoid WebKitGTK's NVIDIA/Wayland DMA-BUF rendering freezes (also in AppImage).
/// Call from main before GTK, WebKit or any application threads are initialized.
pub fn configure_webkit() {
    let wayland_available = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var_os("WAYLAND_SOCKET").is_some()
        || env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");
    let gdk_backend = env::var("GDK_BACKEND").ok();

    if should_disable_dmabuf(
        wayland_available,
        Path::new("/sys/module/nvidia_drm").exists(),
        env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some(),
        gdk_backend.as_deref(),
    ) {
        // Process-local only; explicit user values (including "0") take precedence.
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

fn should_disable_dmabuf(
    wayland_available: bool,
    nvidia_driver_loaded: bool,
    renderer_override_present: bool,
    gdk_backend: Option<&str>,
) -> bool {
    if !wayland_available || !nvidia_driver_loaded || renderer_override_present {
        return false;
    }

    // Respect an explicitly preferred X11 (or other) GTK backend without switching it.
    matches!(
        gdk_backend.and_then(|backends| backends.split(',').next().map(str::trim)),
        None | Some("") | Some("wayland") | Some("*")
    )
}

#[cfg(test)]
mod tests {
    use super::should_disable_dmabuf;

    #[test]
    fn enables_workaround_for_nvidia_on_wayland() {
        for backend in [
            None,
            Some("wayland"),
            Some("wayland,x11"),
            Some(" wayland , x11"),
            Some("*"),
            Some(""),
        ] {
            assert!(
                should_disable_dmabuf(true, true, false, backend),
                "{backend:?}"
            );
        }
    }

    #[test]
    fn leaves_other_environments_unchanged() {
        assert!(!should_disable_dmabuf(false, true, false, None));
        assert!(!should_disable_dmabuf(true, false, false, None));
        for backend in [Some("x11"), Some("x11,wayland"), Some("broadway")] {
            assert!(
                !should_disable_dmabuf(true, true, false, backend),
                "{backend:?}"
            );
        }
    }

    #[test]
    fn preserves_explicit_renderer_override() {
        assert!(!should_disable_dmabuf(true, true, true, None));
        assert!(!should_disable_dmabuf(true, true, true, Some("wayland")));
    }
}
