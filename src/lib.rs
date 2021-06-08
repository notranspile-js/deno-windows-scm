
use std::cell::RefCell;
use std::mem::size_of;
use std::ptr::null_mut;
use std::rc::Rc;
use std::thread;

use deno_core::error::AnyError;
use deno_core::error::generic_error;
use deno_core::op_async;
use deno_core::Extension;
use deno_core::OpState;
use futures::channel::oneshot;
use serde::Deserialize;
use widestring::U16CString;
use winapi::shared::winerror;
use winapi::um::winnt;
use winapi::um::winsvc;
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::LPWSTR;

#[no_mangle]
pub fn init() -> Extension {
  Extension::builder()
    .ops(vec![
      ("op_winscm_start_dispatcher", op_async(op_winscm_start_dispatcher)),
    ])
    .build()
}

#[derive(Deserialize)]
struct Args {
  name: String
}

fn op_error_class(_: &AnyError) -> &'static str {
  return "Error";
}

fn status_str(status: DWORD) -> String {
  match status {
    winsvc::SERVICE_RUNNING => String::from("SERVICE_RUNNING"),
    winsvc::SERVICE_START_PENDING => String::from("SERVICE_START_PENDING"),
    winsvc::SERVICE_STOP_PENDING => String::from("SERVICE_STOP_PENDING"),
    winsvc::SERVICE_STOPPED => String::from("SERVICE_STOPPED"),
    _ => format!("{}", status)
  }
}

unsafe fn set_service_status(ha: winsvc::SERVICE_STATUS_HANDLE, status: DWORD) -> Result<(), AnyError> {
  let mut st = winsvc::SERVICE_STATUS {
    dwServiceType: winnt::SERVICE_WIN32_OWN_PROCESS,
    dwCurrentState: status,
    dwControlsAccepted: winsvc::SERVICE_ACCEPT_STOP | winsvc::SERVICE_ACCEPT_SHUTDOWN,
    dwWin32ExitCode: winerror::NO_ERROR,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: match status {
      winsvc::SERVICE_RUNNING | winsvc::SERVICE_STOPPED => 0,
      _ => 1
    },
    dwWaitHint: 0
  };

  let success = winsvc::SetServiceStatus(ha, &mut st);
  match success {
    0 => Err(generic_error(format!("SetServiceStatus error, status: [{}], code: [{}]", status_str(status), GetLastError()))),
    _ => Ok(())
  }
}

#[no_mangle]
unsafe extern "system"
fn service_control_handler(step: DWORD, _: DWORD, _: LPVOID, ha_ptr: LPVOID) -> DWORD {
  let ha = *(ha_ptr as *mut winsvc::SERVICE_STATUS_HANDLE);
  match step {
    winsvc::SERVICE_CONTROL_STOP | winsvc::SERVICE_CONTROL_SHUTDOWN => {
      match set_service_status(ha, winsvc::SERVICE_STOP_PENDING) {
        Ok(_) => {
          let _ = set_service_status(ha, winsvc::SERVICE_STOPPED);
          ()
        },
        _ => ()
      }
    },
    _ => ()
  };
  winerror::NO_ERROR
}

#[no_mangle]
unsafe extern "system"
fn service_main(_: DWORD, args: *mut LPWSTR) -> () {
  // The first parameter contains the number of arguments being passed to the service in the second parameter.
  // There will always be at least one argument. The second parameter is a pointer to an array of string pointers.
  // The first item in the array is always the service name.
  let name: LPWSTR = *args;
  // this pointer is leaked only once on startup
  let ha_ptr = libc::malloc(size_of::<*mut winsvc::SERVICE_STATUS_HANDLE>()) as *mut winsvc::SERVICE_STATUS_HANDLE;
  *ha_ptr = null_mut();

  // register the handler function for the service
  let ha = winsvc::RegisterServiceCtrlHandlerExW(name, Some(service_control_handler), ha_ptr as LPVOID);
  if  null_mut() == ha {
    // todo: report me somehow
    return ();
  }
  *ha_ptr = ha;
  match set_service_status(ha, winsvc::SERVICE_START_PENDING) {
    Ok(_) => {
      let _ = set_service_status(ha, winsvc::SERVICE_RUNNING);
      ()
    },
    Err(_) => ()
  }
  ()
}

fn start_dispatcher(name: &str) -> Result<(), AnyError> {
  unsafe {
    // call SCM
    let wname = match U16CString::from_str(name) {
      Ok(val) => val,
      Err(_) => {
        return Err(generic_error(format!("Name widen error, value: [{}]", name)))
      }
    };
    let st = vec![
      winsvc::SERVICE_TABLE_ENTRYW {
        lpServiceName: wname.as_ptr(),
        lpServiceProc: Some(service_main)
      },
      winsvc::SERVICE_TABLE_ENTRYW {
        lpServiceName: null_mut(),
        lpServiceProc: None
      }
    ];

    // Connects the main thread of a service process to the service control
    // manager, which causes the thread to be the service control dispatcher
    // thread for the calling process. This call returns when the service has
    // stopped. The process should simply terminate when the call returns.
    match winsvc::StartServiceCtrlDispatcherW(st.as_ptr()) {
      0 => Err(generic_error(format!(
        "StartServiceCtrlDispatcherW error, name: [{}], code: [{}]", name, GetLastError()))),
      _ => Ok(())
    }
  }
}

async fn op_winscm_start_dispatcher(
  state: Rc<RefCell<OpState>>,
  args: Args,
  _: (),
) -> Result<(), AnyError> {
  state.borrow_mut().get_error_class_fn = &op_error_class;

  // spawn thread
  let (tx, rx) = oneshot::channel::<Result<(), AnyError>>();
  thread::spawn(move || {
    let res = start_dispatcher(&args.name);
    tx.send(res).expect("winscm: async op channel send failure");
  });

  // wait for thread to exit
  match rx.await {
    Ok(ok) => ok,
    Err(_) => Err(generic_error("Async op channel receive failure"))
  }
}