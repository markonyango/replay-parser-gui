import { NgComponentOutlet } from '@angular/common';
import { Component } from '@angular/core';
import { ICellRendererAngularComp } from 'ag-grid-angular';
import { ICellRendererParams } from 'ag-grid-community';

@Component({
  standalone: true,
  template: `
    @for (component of params?.components; track $index) {
      <ng-container *ngComponentOutlet="component.component; inputs: { action: component.action, params }"></ng-container>
    }
  `,
  imports: [NgComponentOutlet]
})
export class ActionsCellRendererComponent implements ICellRendererAngularComp {
  protected params: any;

  agInit(params: ICellRendererParams<any, any, any>): void {
    this.params = params;
  }

  refresh(params: ICellRendererParams<any, any, any>): boolean {
    return false;
  }

}
