Windows SCM Deno plugin
=======================

[![appveyor](https://ci.appveyor.com/api/projects/status/github/notranspile-js/deno-windows-scm?svg=true)](https://ci.appveyor.com/project/staticlibs/deno-windows-scm)

Native FFI library for [Deno](https://deno.land/) that allows to control scripts from
Windows [Service Control Manager](https://docs.microsoft.com/en-us/windows/win32/services/service-control-manager).

Usage example
-------------

Create SCM service using [sc tool](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/sc-create), run from console with Administrator privileges:

```
sc create myservice binpath="path\to\deno.exe run --unstable --allow-ffi --allow-read --allow-write path\to\script.js"
```

In `script.js`:

```
import { winscmStartDispatcher } from "https://raw.githubusercontent.com/notranspile-js/deno-windows-scm/<version>/ts/mod.ts";

// this promise resolves when the service is stopped
await winscmStartDispatcher({
    libraryPath: "path/to/deno_windows_scm_<version>.dll",
    serviceName: "myservice",
    logFilePath: "path/to/scm_log.txt", // optional, for troubleshooting
});
```

License information
-------------------

This project is released under the [Apache License 2.0](http://www.apache.org/licenses/LICENSE-2.0).