import { Injectable } from "@angular/core";
import { MatSnackBar } from "@angular/material/snack-bar";

import { checkUpdate, installUpdate } from '@tauri-apps/api/updater';
import { relaunch } from '@tauri-apps/api/process';
import { getVersion } from '@tauri-apps/api/app';
import { switchMap } from "rxjs";

@Injectable({ providedIn: 'root' })
export class UpdateService {
  constructor(private _snackBar: MatSnackBar) { }

  async check() {
    const { shouldUpdate, manifest } = await checkUpdate();
    const currentVersion = await getVersion();

    if (shouldUpdate) {
      this._snackBar.open(`Update available (${currentVersion} -> ${manifest?.version}): ${manifest?.body}`, 'Update')
        .afterDismissed()
        .pipe(
          switchMap(() => installUpdate()),
          switchMap(() => relaunch())
        )
        .subscribe({
          error: error => this._snackBar.open(`Update failed: ${error ?? 'unknown error'}`, 'Dismiss'),
        });
    }
  }
}
