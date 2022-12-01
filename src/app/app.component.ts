import { DOCUMENT } from '@angular/common';
import { Component, Inject } from '@angular/core';
import { appWindow } from '@tauri-apps/api/window';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent {
  public constructor() {
    appWindow.listen('channel', console.log).then().catch(console.error);
  }
}
