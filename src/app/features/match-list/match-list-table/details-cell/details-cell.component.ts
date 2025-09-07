import { Component, inject, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { ICellRendererAngularComp } from 'ag-grid-angular';
import { ICellRendererParams } from 'ag-grid-community';
import { MatchDetailsComponent } from '../match-details.component';

@Component({
  template: `
        <button mat-icon-button ><mat-icon (click)="openDialog()">info</mat-icon></button>
    `,
  imports: [MatButtonModule, MatIconModule, MatDialogModule],
  standalone: true,
})
export class DetailsCellComponent implements ICellRendererAngularComp {
  private _dialog = inject(MatDialog);

  agInit(params: ICellRendererParams<any, any, any>): void {
    this.data.set(params.data);
  }

  refresh(params: ICellRendererParams<any, any, any>): boolean {
    return false;
  }

  data = signal<any>(undefined);

  openDialog() {
    this._dialog.open(MatchDetailsComponent, {
      hasBackdrop: true, data: this.data(), height: '80vh', minWidth: '80vw'
    })
  }
}
