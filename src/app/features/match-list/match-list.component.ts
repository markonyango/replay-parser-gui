import { Component, inject } from '@angular/core';
import { MatchListTableComponent } from './match-list-table/match-list-table.component';
import { TauriService } from '../../core/services/tauri.service';

@Component({
  selector: 'app-match-list',
  templateUrl: './match-list.component.html',
  styleUrls: ['./match-list.component.css'],
  standalone: true,
  imports: [MatchListTableComponent]
})
export class MatchListComponent {
  private _matchListService = inject(TauriService);

  matchList = this._matchListService.matchList;
}
