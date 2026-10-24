import { Component, inject } from '@angular/core';
import { UpdateService } from './core/services/update.service';
import { RouterModule } from '@angular/router';
import { StatusBarComponent } from './core/components/status-bar/status-bar.component';
import { StatusService } from './core/services/status.service';
import { TauriService } from './core/services/tauri.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
  imports: [RouterModule, StatusBarComponent],
})
export class AppComponent {
  private _updateService = inject(UpdateService);
  protected readonly status = inject(StatusService);
  // Instantiated here so the backend status listener is registered at startup.
  private _tauriService = inject(TauriService);

  ngOnInit() {
    this._updateService.check();
  }
}
