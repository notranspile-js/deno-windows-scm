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

import existsSync from "./existsSync.ts";
import { WinSCMOptions, WorkerRequest, WorkerResponse } from "./types.ts";

let workerStarted = false;

/**
 * Creates a Worker and connects its thread to the Service Control Manager,
 * which causes the thread to be the service control dispatcher thread.
 * 
 * Can be called only once in a single process.
 *
 * ```ts
 * await winscmStartDispatcher({
 *  libraryPath: "path/to/deno_windows_scm_<version>.dll",
 *  serviceName: "myservice",
 *  logFilePath: "path/to/scm_log.txt", // optional, for troubleshooting
 * })
 * ```
 *
 * @param opts path to the native library and a name of the service
 * @return a promise, that resolves when the Windows Service is stopped
 */
export default async function winscmStartDispatcher(opts: WinSCMOptions) {
  if (workerStarted) throw new Error("SCM worker is already started");
  if (!existsSync(opts.libraryPath)) {
    throw new Error(
      `Native library not found, specified path: [${opts.libraryPath}]`,
    );
  }
  if (
    !("string" === typeof (opts.serviceName) && opts.serviceName.length > 0)
  ) {
    throw new Error(
      "WinSCM service name must be specified",
    );
  }

  const workerPath = new URL("./worker.ts", import.meta.url).href;
  const worker = new Worker(workerPath, {
    type: "module",
    name: opts.workerName ?? "WinSCMWorker",
    deno: {
      namespace: true,
    },
  });
  workerStarted = true;

  const promise = new Promise((resolve, reject) => {
    worker.onmessage = (e: MessageEvent) => {
      const res: WorkerResponse = JSON.parse(e.data);
      if (true === res.success) {
        resolve(null);
      } else {
        reject(res.error ?? "N/A");
      }
    };
  });

  const workerRequest: WorkerRequest = {
    libraryPath: opts.libraryPath,
    serviceName: opts.serviceName,
    logFilePath: opts.logFilePath ?? ""
  };
  worker.postMessage(JSON.stringify(workerRequest));

  await promise;
}