import { ApplicationConfig, provideZoneChangeDetection } from "@angular/core";
import { provideRouter, Routes, withViewTransitions } from "@angular/router";

export const routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    loadChildren: () => import('./features/match-list/match-list.module').then(m => m.MatchListModule),
  }
];

export const appConfig: ApplicationConfig = {
  providers: [
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes, withViewTransitions()),
  ]
};

