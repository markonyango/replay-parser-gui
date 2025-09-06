import { Injectable } from "@angular/core";
import { MatLegacySnackBar as MatSnackBar } from "@angular/material/legacy-snack-bar";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { switchMap } from "rxjs";
@Injectable({ providedIn: "root" })
export class UpdateService {
  constructor(private _snackBar: MatSnackBar) { }
  async check() {
    const update = await check();
    const currentVersion = await getVersion();
    if (update) {
      this._snackBar.open(
        `Update available (${currentVersion} -> ${update?.version}): ${update?.body}`,
        "Update",
      )
        .afterDismissed()
        .pipe(
          switchMap(() => update.downloadAndInstall()),
          switchMap(() => relaunch()),
        )
        .subscribe({
          error: (error) =>
            this._snackBar.open(
              `Update failed: ${error ?? "unknown error"}`,
              "Dismiss",
            ),
        });
    }
  }
}
