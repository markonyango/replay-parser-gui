import { Component } from '@angular/core';
import { TauriService } from 'src/app/core/services/tauri.service';

@Component({
  selector: 'app-match-list',
  templateUrl: './match-list.component.html',
  styleUrls: ['./match-list.component.css'],
})
export class MatchListComponent {
  matchList$ = this._matchListService.matchList$;

  public constructor(private _matchListService: TauriService) {}
}
