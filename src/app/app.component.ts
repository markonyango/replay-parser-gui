import { Component, inject } from '@angular/core';
import { UpdateService } from './core/services/update.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
  standalone: false
})
export class AppComponent {
  private _updateService = inject(UpdateService);

  ngOnInit() {
    this._updateService.check();
  }
}
