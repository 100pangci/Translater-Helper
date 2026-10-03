use std::{env, path::Path};

/// Avoid WebKitGTK's NVIDIA DMA-BUF rendering freezes, including XWayland AppImage.
/// Call from main before GTK, WebKit or any application threads are initialized.
pub fn configure_webkit() {
    if should_disable_dmabuf(
        Path::new("/sys/module/nvidia_drm").exists(),
        env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some(),
    ) {
        // Process-local only; explicit user values (including "0") take precedence.
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

fn should_disable_dmabuf(nvidia_driver_loaded: bool, renderer_override_present: bool) -> bool {
    // linuxdeploy's AppImage GTK hook forces GDK_BACKEND=x11, even on Wayland.
    // Do not gate this workaround on the backend: that would skip AppImage entirely.
    nvidia_driver_loaded && !renderer_override_present
}

#[cfg(test)]
mod tests {
    use super::should_disable_dmabuf;

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
}
