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

// import { SCMConfig, WorkerRequest, WorkerResponse, WorkerSelf } from "./types.ts";

function createErrorMessage(req /*: WorkerRequest */, code /*: number */) /*: string */ {
  if (code < 0) {
    return "WinSCM call initialization failure, details unavailable";
  }
  if (0 === req.logFilePath.length) {
    return "WinSCM call failure, 'logFilePath' option can be supplied for detailed logging";
  }
  try {
    const details = Deno.readTextFileSync(req.logFilePath);
    return `WinSCM call failure, details:\n${details}`;
  } catch (_) {
    return "WinSCM call system failure, details unavailable";
  }
}

self.onmessage = (e /*: MessageEvent */) => {
  let resp /*: WorkerResponse | null */ = null;
  try {
    const req /*: WorkerRequest */ = JSON.parse(e.data);

    const dylib = Deno.dlopen(req.libraryPath, {
      "winscm_start_dispatcher": {
        parameters: [],
        result: "i32",
      },
    });

    const confPath = `${req.libraryPath}.config.json`;
    const config /*: SCMConfig */ = {
      serviceName: req.serviceName,
      logFilePath: req.logFilePath,
    };
    Deno.writeTextFileSync(confPath, JSON.stringify(config, null, 4));

    const code = /* <number> */ dylib.symbols.winscm_start_dispatcher();
    if (0 !== code) {
      throw new Error(createErrorMessage(req, code));
    }
    resp = {
      success: true,
    };
  } catch (e) {
    resp = {
      success: false,
      error: String(e),
    };
  } finally {
    self.postMessage(JSON.stringify(resp));
    self.close();
  }
};
