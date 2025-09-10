import { Component, input } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';

@Component({
  standalone: true,
  template: '<button mat-icon-button><mat-icon (click)="action()(params())">delete</mat-icon></button>',
  imports: [MatButtonModule, MatIconModule]
})
export class DeleteRendererComponent {
  action = input.required<any>();
  params = input();
}
