// init signals
  unsafe {
    let mask: *mut libc::sigset_t = libc::malloc(std::mem::size_of::<libc::sigset_t>()) as *mut libc::sigset_t;
    let err_fill = libc::sigfillset(mask);
    if 0 != err_fill {
      return Err(generic_error(format!("Error initializing signals: [{}]", err_fill)))
    }
    let err_mask = libc::pthread_sigmask(libc::SIG_SETMASK, mask, std::ptr::null_mut());
    if 0 != err_mask {
      return Err(generic_error(format!("Error initializing signals mask: [{}]", err_mask)))
    }
    // mask is leaked here
  }

  unsafe {
    // notify systemd
    let pid = std::process::id();
    let state = std::ffi::CString::new("READY=1").unwrap();
    let err_notify = sd_pid_notify(pid as libc::pid_t, 0, state.as_ptr());
    if err_notify <= 0 {
      return Err(generic_error(format!("SystemD notify failure, code: [{}]", err_notify)));
    }

    // wait for signals, minor race condition is here
    let mask: *mut libc::sigset_t = libc::malloc(std::mem::size_of::<libc::sigset_t>()) as *mut libc::sigset_t;
    let err_empty = libc::sigemptyset(mask);
    if 0 != err_empty {
      return Err(generic_error(format!("Error initializing signals handler, code: [{}]", err_empty)));
    }
    let err_int = libc::sigaddset(mask, libc::SIGINT);
    if 0 != err_int {
      return Err(generic_error(format!("Error initializing signals handler, code: [{}]", err_int)));
    }
    let err_term = libc::sigaddset(mask, libc::SIGTERM);
    if 0 != err_term {
      return Err(generic_error(format!("Error initializing signals handler, code: [{}]", err_term)));
    }

    println!("Is due to wait for signals ...");
    // wait for signal
    let mut sig: libc::c_int = -1;
    let err_wait = libc::sigwait(mask, &mut sig);
    println!("Incoming signal!");
    if 0 != err_wait {
      return Err(generic_error(format!("Error waiting for signal, code: [{}]", err_wait)));
    }
    // mask is leaked here