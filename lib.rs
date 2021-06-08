
extern crate libc;

use std::cell::RefCell;
use std::rc::Rc;

use deno_core::error::AnyError;
use deno_core::error::generic_error;
use deno_core::op_async;
use deno_core::Extension;
use deno_core::OpState;
use serde::Deserialize;

#[no_mangle]
pub fn init() -> Extension {
  Extension::builder()
    .ops(vec![
      ("op_osservice_run", op_async(op_osservice_run)),
    ])
    .build()
}

#[derive(Deserialize)]
struct Args {
}

fn op_error_class(_: &AnyError) -> &'static str {
  return "Error";
}

#[cfg(target_os = "linux")]
#[link(name = "systemd", kind = "dylib")]
extern "C" {
  fn sd_pid_notify(pid: libc::pid_t, unset_environment: libc::c_int, state: *const libc::c_char) -> libc::c_int;
}

#[cfg(target_os = "linux")]
async fn op_osservice_run(
  state: Rc<RefCell<OpState>>,
  _args: Option<Args>,
  _: (),
) -> Result<(), AnyError> {
  state.borrow_mut().get_error_class_fn = &op_error_class;

  // spawn thread
  let (tx, rx) = futures::channel::oneshot::channel::<Result<(), AnyError>>();
  std::thread::spawn(move || {
    let err_notify = unsafe {
      // notify systemd
      let pid = std::process::id();
      let state = std::ffi::CString::new("READY=1").unwrap();
      sd_pid_notify(pid as libc::pid_t, 0, state.as_ptr())
    };
    let res = if err_notify > 0 {
      Ok(())
    } else {
      Err(generic_error(format!("Error notifying SystemD, code: [{}]", err_notify)))
    };
    tx.send(res).expect("osservice: async op channel send failure");
  });

  // wait for thread to exit
  match rx.await {
    Ok(ok) => ok,
    Err(_) => Err(generic_error("Async op channel receive failure"))
  }
}
