use phantom_dependencies::{
    anyhow::{bail, Result},
    gl,
    glutin::{platform::windows::RawContextExt, ContextBuilder, ContextWrapper, PossiblyCurrent},
    raw_window_handle::{HasRawWindowHandle, RawWindowHandle},
};

pub unsafe fn load_context(
    window_handle: &impl HasRawWindowHandle,
) -> Result<ContextWrapper<PossiblyCurrent, ()>> {
    let raw_context = match window_handle.raw_window_handle() {
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(handle) => {
            ContextBuilder::new().build_raw_context(handle.hwnd as _)?
        }

        #[cfg(target_os = "unix")]
        RawWindowHandle::Xlib(handle) => {
            ContextBuilder::new().build_raw_context(handle.display as _)?
        }

        _ => bail!("The target operating system is not supported!"),
    };

    let context = raw_context.make_current().unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    Ok(context)
}
