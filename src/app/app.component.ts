import { Component } from '@angular/core';
import { UpdateService } from './core/services/update.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
})
export class AppComponent {
  constructor(private _updateService: UpdateService) {}
  ngOnInit() {
    this._updateService.check();
  }
}
