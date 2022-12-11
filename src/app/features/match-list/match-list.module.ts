import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';

import { MatchListComponent } from './match-list.component';
import { MatchListTableComponent } from './match-list-table/match-list-table.component';
import { PushModule } from '@rx-angular/template';
import { MatchListRoutingModule } from './match-list-routing.module';


@NgModule({
  declarations: [MatchListComponent, MatchListTableComponent],
  imports: [CommonModule, MatchListRoutingModule, MatTableModule, PushModule],
  exports: [MatchListComponent],
})
export class MatchListModule {}
