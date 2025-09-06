import { Component, signal } from '@angular/core';
import { MatIconModule } from '@angular/material/icon';
import { ICellRendererAngularComp } from 'ag-grid-angular';
import { ICellRendererParams } from 'ag-grid-community';

@Component({
    template: `
        @if (ok()) {
            <mat-icon>check</mat-icon>
        } @else {
            <mat-icon>warning</mat-icon>
        }
    `,
    imports: [MatIconModule],
    standalone: true,
})
export class StatusCellComponent implements ICellRendererAngularComp {
    agInit(params: ICellRendererParams<any, any, any>): void {
        console.log(params.value);
        if (params.value?.status) {
            const status = JSON.parse(params.value?.status);
            this.ok.set(status.response == 'ok');
        }
    }

    refresh(params: ICellRendererParams<any, any, any>): boolean {
        return false;
    }

    ok = signal<boolean>(false);
}
