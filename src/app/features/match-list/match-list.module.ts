import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';
import { MatIconModule } from '@angular/material/icon';

import { MatchListComponent } from './match-list.component';
import { MatchListTableComponent } from './match-list-table/match-list-table.component';
import { MatchListRoutingModule } from './match-list-routing.module';
import { PlayersCellComponent } from './match-list-table/players-cell/players-cell.component';
import { MapCellComponent } from './match-list-table/map-cell/map-cell.component';

@NgModule({
  declarations: [
    MatchListComponent,
    MatchListTableComponent,
    PlayersCellComponent,
    MapCellComponent,
  ],
  imports: [
    CommonModule,
    MatchListRoutingModule,
    MatIconModule,
    MatTableModule,
  ],
  exports: [MatchListComponent],
})
export class MatchListModule {}
