use std::any::Any;
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn set_last_error(msg: impl Into<String>) {
    let msg = msg.into();
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = Some(msg);
    });
}

pub(crate) fn panic_msg(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        format!("panic: {s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("panic: {s}")
    } else {
        "panic: unknown".to_string()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_last_error() -> *mut c_char {
    let msg = LAST_ERROR.with(|cell| cell.borrow_mut().take());
    match msg {
        Some(s) => match CString::new(s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}
