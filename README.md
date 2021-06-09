Windows SCM Deno plugin
=======================

[![appveyor](https://ci.appveyor.com/api/projects/status/github/notranspile-js/deno-windows-scm?svg=true)](https://ci.appveyor.com/project/staticlibs/deno-windows-scm)

Native plugin for [Deno](https://deno.land/) that allows to control scripts from
Windows [Service Control Manager](https://docs.microsoft.com/en-us/windows/win32/services/service-control-manager).

Usage example
-------------

Create SCM service using [sc tool](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/sc-create), run from console with Administrator privileges:

```
sc create myservice binpath="path\to\deno.exe run --unstable --allow-plugin path\to\script.js"
```

In `script.js`:

```
Deno.openPlugin("path/to/windows_scm_plugin_<version>.dll");

// this call blocks until the service is stopped
await Deno.core.opAsync("op_winscm_start_dispatcher", {
    name: "myservice"
});
```

License information
-------------------

This project is released under the [Apache License 2.0](http://www.apache.org/licenses/LICENSE-2.0).