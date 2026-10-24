import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';
import { AgGridAngular } from 'ag-grid-angular';
import { ColDef } from 'ag-grid-community';
import { ticks2time } from 'src/app/core/services/tauri.service';
import { ActionInfo, MatchItem, MessageInfo } from 'src/types';

@Component({
    template: `
        <h1 mat-dialog-title>Match Details ({{ data.match_id }})</h1>
        <mat-dialog-content>
            <div class="flex flex-col gap-5">
                <h2>Messages</h2>
                <ag-grid-angular
                    style="width: 100%; height: 400px"
                    [rowData]="messages"
                    [columnDefs]="messages_colDefs"
                />
                <h2>Actions</h2>
                <ag-grid-angular
                    style="width: 100%; height: 400px"
                    [rowData]="actions"
                    [columnDefs]="actions_colDefs"
                />
            </div>
        </mat-dialog-content>
        <mat-dialog-actions align="end">
            <button matButton mat-dialog-close>Close</button>
        </mat-dialog-actions>
    `,
    imports: [AgGridAngular, MatDialogModule, MatCardModule, MatButtonModule],
})
export class MatchDetailsComponent {
    protected data = inject<MatchItem>(MAT_DIALOG_DATA);

    protected messages = this.data.messages;
    protected actions = this.data.actions;

    protected messages_colDefs: ColDef<MessageInfo>[] = [
        {
            flex: 0.2,
            field: 'tick',
            valueFormatter: (params) => ticks2time(params.data?.tick ?? 0),
        },
        { flex: 0.2, field: 'sender' },
        { flex: 0.2, field: 'receiver' },
        { flex: 1, field: 'body' },
    ];

    protected actions_colDefs: ColDef<ActionInfo>[] = [
        { flex: 1, field: 'relic_id', headerName: 'Relic ID' },
        { flex: 1, field: 'name', filter: 'agTextColumnFilter' },
        {
            flex: 1,
            field: 'tick',
            valueFormatter: (params) => ticks2time(params.data?.tick ?? 0),
        },
        {
            flex: 1,
            field: 'data',
            headerName: 'Action',
            valueFormatter: (params) =>
                (params.data as any)?.command.opcode_name,
        },
        {
            flex: 1,
            field: 'data',
            headerName: 'Target',
            valueFormatter: (params) =>
                (params.data as any)?.command?.blueprint?.display_name ?? '',
        },
    ];
}
