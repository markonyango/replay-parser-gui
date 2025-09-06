import { NgModule } from '@angular/core';

import { MatchListComponent } from './match-list.component';
import { MatchListRoutingModule } from './match-list-routing.module';

@NgModule({
  imports: [
    MatchListComponent,
    MatchListRoutingModule,
  ],
  exports: [MatchListComponent],
})
export class MatchListModule { }
