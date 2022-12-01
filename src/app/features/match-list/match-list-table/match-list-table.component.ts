import { ChangeDetectionStrategy, Component, Input } from '@angular/core';

@Component({
  selector: 'app-match-list-table',
  templateUrl: './match-list-table.component.html',
  styleUrls: ['./match-list-table.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MatchListTableComponent {
  @Input() public dataSource: any[] = [];

  columnsToDisplay = [
    'match_id',
    'players',
    'map',
    'duration',
    'status',
    'played_at',
  ];
}
