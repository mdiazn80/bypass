// Tauri's beforeBuildCommand. CI sets CI_REUSE_FRONTEND after the checks step
// has already produced dist/, so the release bundle does not build it again.
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";

if (process.env.CI_REUSE_FRONTEND === "1") {
  if (!existsSync("dist/index.html")) {
    console.error("CI_REUSE_FRONTEND=1 but dist/index.html is missing");
    process.exit(1);
  }
  console.log("Reusing dist/ from the checks step");
  process.exit(0);
}

const result = spawnSync("pnpm", ["build"], { stdio: "inherit", shell: true });
process.exit(result.status === null ? 1 : result.status);
