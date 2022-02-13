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

/**
 * Options object to pass to `winscmStartDispatcher`
 */
export type WinSCMOptions = {
  /**
   * absolute path to the native library deno_windows_scm_<version>.dll
   */
  libraryPath: string,
  /**
   * name of the Windows Service that was previously registered
   */
  serviceName: string,
  /**
   * absolute path to the local copy of a 'winscmWorker.js' file
   */
  workerPath: string,
  /**
   * optional, native library will use this file for logging, can be useful for troubleshooting
   */
  logFilePath?: string,
  /**
   * optional, the name of the worker that is created by `winscmStartDispatcher`
   */
  workerName?: string,
}

export type WorkerRequest = {
  libraryPath: string,
  serviceName: string,
  logFilePath: string,
}

export type WorkerResponse = {
  success: boolean,
  error?: string
}

export type SCMConfig = {
  serviceName: string,
  logFilePath: string
}