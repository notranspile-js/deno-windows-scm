/*
 * Copyright 2017, alex at staticlibs.net
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

// sc create foo binpath="path\to\deno.exe run --unstable --allow-plugin --allow-read --allow-write path\to\test.js path\to\windows_scm_plugin.dll"

const filename = Deno.args[0]

Deno.openPlugin(filename);
const path = import.meta.url.substring("file:///".length);

Deno.writeTextFileSync(path + ".1.txt", "is due to start service");

await Deno.core.opAsync("op_winscm_start_dispatcher", {
    name: "foo"
});

Deno.writeTextFileSync(path + ".2.txt", "stopping service");