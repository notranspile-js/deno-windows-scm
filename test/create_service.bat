@echo off

set BAD_SLASH_SCRIPT_DIR=%~dp0
set SCRIPT_DIR=%BAD_SLASH_SCRIPT_DIR:\=/%

if "x%DENO_HOME%" == "x" (
    echo Error: DENO_HOME environment variable must be set
    exit /b 1
)

set deno_exe="%DENO_HOME%"\deno.exe
set deno_options=--unstable --allow-ffi --allow-read --allow-write
set entry_point_script=%SCRIPT_DIR%scmEntryPoint.ts

sc create deno_windows_scm_test ^
    binpath="%deno_exe% run %deno_options% %entry_point_script%"

echo Service installed, to delete it run: "sc stop deno_windows_scm_test && sc delete deno_windows_scm_test"