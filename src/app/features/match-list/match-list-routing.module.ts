import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { MatchListComponent } from './match-list.component';

const routes: Routes = [
  {
    path: '',
    component: MatchListComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class MatchListRoutingModule {}
