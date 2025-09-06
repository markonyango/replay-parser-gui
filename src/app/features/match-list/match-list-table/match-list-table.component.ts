import {
  trigger,
  state,
  style,
  transition,
  animate,
} from '@angular/animations';
import { ChangeDetectorRef, Component, inject, input } from '@angular/core';
import { ReplayInfo } from 'src/types';

@Component({
  selector: 'app-match-list-table',
  templateUrl: './match-list-table.component.html',
  styleUrls: ['./match-list-table.component.css'],
  animations: [
    trigger('detailExpand', [
      state('collapsed', style({ height: '0px', minHeight: '0' })),
      state('expanded', style({ height: '*' })),
      transition('expanded <=> collapsed', animate('225ms cubic-bezier(0.4, 0.0, 0.2, 1)')),
    ]),
  ],
  standalone: false
})
export class MatchListTableComponent {
  dataSource = input<Partial<ReplayInfo>[]>([]);

  expandedElement: ReplayInfo | null = null;

  columnsToDisplay = [
    'match_id',
    'players',
    'map',
    'duration',
    'status',
    'played_at',
  ];

  columnsToDisplayWithExpand = [...this.columnsToDisplay, 'expand'];

  private cdr = inject(ChangeDetectorRef);

  handleRowClick(element: ReplayInfo) {
    this.expandedElement = this.expandedElement == element ? null : element;
    this.cdr.detectChanges();
  }
}
