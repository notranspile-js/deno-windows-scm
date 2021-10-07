/*
 * Copyright 2021, alex at staticlibs.net
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::error;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::mem::size_of;
use std::mem::transmute;
use std::ptr::null_mut;

use chrono;
use serde_derive::Deserialize;
use serde_json;
use widestring::U16CStr;
use widestring::U16CString;
use winapi::shared::winerror;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi;
use winapi::um::winnt;
use winapi::um::winsvc;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::HMODULE;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::ntdef::LPWSTR;
use std::io::Write;

#[derive(Debug, Clone)]
struct SCMError {
  msg: String,
  code: DWORD
}

impl SCMError {
  fn new(msg: String, code: DWORD) -> SCMError {
      SCMError { msg, code }
  }
}

impl fmt::Display for SCMError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WinSCM error, code: [{}], message: [{}]", self.code, &self.msg)
  }
}

impl error::Error for SCMError { }

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct SCMConfig {
  serviceName: String,
  logFilePath: String
}

fn get_dll_path(max_size: u16) -> Result<String, SCMError>  {
  unsafe {
    let mut hm: HMODULE = null_mut();

    let res_mh = libloaderapi::GetModuleHandleExW(
      libloaderapi::GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
          libloaderapi::GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
      transmute::<*const (), LPCWSTR>(winscm_start_dispatcher as *const ()),
      &mut hm as *mut HMODULE);
    if 0 == res_mh {
      return Err(SCMError::new(
        "GetModuleHandleExW error".to_string(), GetLastError()));
    }

    let mut vec: Vec<u16> = vec![0; max_size as usize];
    let ptr: LPWSTR = vec.as_mut_ptr();
    let len = libloaderapi::GetModuleFileNameW(hm, ptr, vec.len() as DWORD);
    if !(len > 0 && len < max_size.into()) {
      return Err(SCMError::new(format!(
        "GetModuleFileNameW error, returned length: [{}]", len), GetLastError()));
    }

    let cstr = U16CStr::from_ptr_with_nul(ptr, len as usize);
    match cstr.to_string() {
      Ok(str) => Ok(str),
      Err(e) => Err(SCMError::new(format!(
        "GetModuleFileNameW UTF-8 error, message: [{}]", e), 1))
    }
  }
}

fn read_config() -> Result<SCMConfig, SCMError> {
  let dll_path = get_dll_path(1024)?;
  let config_path = dll_path + ".config.json";
  let json = match fs::read_to_string(&config_path) {
    Ok(json) => json,
    Err(e) => return Err(SCMError::new(format!(
      "Error reading config file, path: [{}], message: [{}]", &config_path, e.to_string()
    ), e.raw_os_error().unwrap_or(1) as u32))
  };
  match serde_json::from_str(&json) {
    Ok(conf) => Ok(conf),
    Err(e) => return Err(SCMError::new(format!(
      "Error deserializing config file, path: [{}], message: [{}]", &config_path, e.to_string()
    ), 1))
  }
}

fn write_to_log(conf: &SCMConfig, msg: String) -> () {
  if conf.logFilePath.is_empty() {
    return;
  }
  match OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(&conf.logFilePath) {
    Err(_) => (),
    Ok(mut file) => {
      let tm = chrono::Local::now();
      let tm_formatted = tm.to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
      let _ = file.write_all(format!("{} {}\r\n", tm_formatted, msg).as_bytes());
    }
  }
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

unsafe fn set_service_status(ha: winsvc::SERVICE_STATUS_HANDLE, status: DWORD) -> Result<(), SCMError> {
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
    0 => Err(SCMError::new(format!(
      "SetServiceStatus error, status: [{}]", status_str(status)), GetLastError())),
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
    // note: it may be better to report it somehow
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

fn start_dispatcher(name: &str) -> Result<(), SCMError> {
  unsafe {
    // call SCM
    let wname = match U16CString::from_str(name) {
      Ok(val) => val,
      Err(_) => {
        return Err(SCMError::new(format!(
          "Name widen error, value: [{}]", name), 1))
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
      0 => Err(SCMError::new(format!(
        "StartServiceCtrlDispatcherW error, name: [{}]", name), GetLastError())),
      _ => Ok(())
    }
  }
}

#[no_mangle]
pub extern "C"
fn winscm_start_dispatcher() -> i32 {

  let conf = match read_config() {
    Ok(conf) => conf,
    Err(_) => return -1
  };

  if !conf.logFilePath.is_empty() {
    let _ = fs::remove_file(&conf.logFilePath);
  }

  write_to_log(&conf, format!("Is due to call SCM dispatcher, service name: [{}]", &conf.serviceName));

  match start_dispatcher(&conf.serviceName) {
    Ok(_) => {
      write_to_log(&conf, "SCM dispatcher run complete, service is stopping now".to_string());
      0
    },
    Err(e) => {
      write_to_log(&conf, format!("SCM dispatcher error, code: [{}], message: [{}]", e.code, &e.msg));
      1
    }
  }
}