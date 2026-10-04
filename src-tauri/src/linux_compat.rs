use std::{
    env,
    ffi::{OsStr, OsString},
    path::Path,
};

/// Configure GTK/WebKit before initialization, not after the first window is created.
/// Call from main before GTK, WebKit or any application threads are initialized.
pub fn configure_webkit() {
    let nvidia_driver_loaded = Path::new("/sys/module/nvidia_drm").exists();
    let wayland_available = ["WAYLAND_DISPLAY", "WAYLAND_SOCKET"]
        .iter()
        .any(|name| env::var_os(name).is_some_and(|value| !value.is_empty()));
    let in_appimage = env::var_os("APPIMAGE").is_some() || env::var_os("APPDIR").is_some();
    let backend_override = env::var_os("TRANSHELPER_GDK_BACKEND");
    if let Some(backend) = preferred_backend(
        nvidia_driver_loaded,
        wayland_available,
        in_appimage,
        backend_override.as_deref(),
    ) {
        env::set_var("GDK_BACKEND", backend);
    }

    let dmabuf_override = env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER");
    if should_disable_dmabuf(nvidia_driver_loaded, dmabuf_override.is_some()) {
        // Process-local only; explicit user values (including "0") take precedence.
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    if should_disable_compositing(
        nvidia_driver_loaded,
        dmabuf_override.as_deref(),
        env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_some(),
    ) {
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
}

fn preferred_backend(
    nvidia_driver_loaded: bool,
    wayland_available: bool,
    in_appimage: bool,
    backend_override: Option<&OsStr>,
) -> Option<OsString> {
    if let Some(backend) = backend_override.filter(|value| !value.is_empty()) {
        return Some(backend.to_os_string());
    }
    // The AppImage GTK hook forces X11. On NVIDIA + Wayland this can leave the
    // displayed frame stale while clicks/typing still update the underlying DOM.
    // Prefer native Wayland; keep X11 as a fallback when no compositor is reachable.
    (nvidia_driver_loaded && wayland_available && in_appimage)
        .then(|| OsString::from("wayland,x11"))
}

fn should_disable_dmabuf(nvidia_driver_loaded: bool, renderer_override_present: bool) -> bool {
    // linuxdeploy's AppImage GTK hook forces GDK_BACKEND=x11, even on Wayland.
    // Do not gate this workaround on the backend: that would skip AppImage entirely.
    nvidia_driver_loaded && !renderer_override_present
}

fn should_disable_compositing(
    nvidia_driver_loaded: bool,
    dmabuf_override: Option<&OsStr>,
    compositing_override_present: bool,
) -> bool {
    // Preserve the previous renderer opt-out: an explicit DMA-BUF value other
    // than "1" must not silently turn on a second software-rendering switch.
    nvidia_driver_loaded
        && !compositing_override_present
        && (dmabuf_override.is_none() || dmabuf_override == Some(OsStr::new("1")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_native_wayland_for_nvidia_appimage() {
        assert_eq!(
            preferred_backend(true, true, true, None),
            Some(OsString::from("wayland,x11"))
        );
    }

    #[test]
    fn leaves_other_backends_and_drivers_unchanged() {
        assert_eq!(preferred_backend(false, true, true, None), None);
        assert_eq!(preferred_backend(true, false, true, None), None);
        assert_eq!(preferred_backend(true, true, false, None), None);
    }

    #[test]
    fn preserves_application_backend_override() {
        assert_eq!(
            preferred_backend(true, true, true, Some(OsStr::new("x11"))),
            Some(OsString::from("x11"))
        );
        assert_eq!(
            preferred_backend(false, false, false, Some(OsStr::new("wayland"))),
            Some(OsString::from("wayland"))
        );
    }

    #[test]
    fn enables_workaround_for_nvidia_regardless_of_gtk_backend() {
        assert!(should_disable_dmabuf(true, false));
    }

    #[test]
    fn leaves_other_drivers_unchanged() {
        assert!(!should_disable_dmabuf(false, false));
        assert!(!should_disable_dmabuf(false, true));
    }

    #[test]
    fn preserves_explicit_renderer_override() {
        assert!(!should_disable_dmabuf(true, true));
    }

    #[test]
    fn enables_software_compositing_for_nvidia() {
        assert!(should_disable_compositing(true, None, false));
        assert!(should_disable_compositing(
            true,
            Some(OsStr::new("1")),
            false
        ));
        assert!(!should_disable_compositing(false, None, false));
    }

    #[test]
    fn preserves_renderer_and_compositing_opt_out() {
        assert!(!should_disable_compositing(
            true,
            Some(OsStr::new("0")),
            false
        ));
        assert!(!should_disable_compositing(
            true,
            Some(OsStr::new("")),
            false
        ));
        assert!(!should_disable_compositing(true, None, true));
    }
}
