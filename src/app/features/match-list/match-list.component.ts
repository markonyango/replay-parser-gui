import { Component } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TauriService } from 'src/app/core/services/tauri.service';

@Component({
    selector: 'app-match-list',
    templateUrl: './match-list.component.html',
    styleUrls: ['./match-list.component.css'],
    standalone: false
})
export class MatchListComponent {
  matchList = toSignal(this._matchListService.matchList$);

  public constructor(private _matchListService: TauriService) {}
}
