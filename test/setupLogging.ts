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

import { Logger, log } from "./deps.ts";
import { testLogPath } from "./paths.ts";

export default async (): Promise<Logger> => {
  await log.setup({
    handlers: {
      file: new log.handlers.FileHandler("DEBUG", {
        filename: testLogPath,
        mode: "w",
      }),
    },

    loggers: {
      default: {
        level: "DEBUG",
        handlers: ["file"],
      },
    },
  });

  return log.getLogger();
};
