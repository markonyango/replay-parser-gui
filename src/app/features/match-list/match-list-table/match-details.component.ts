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
        <ag-grid-angular style="width: 100%; height: 400px" [rowData]="messages" [columnDefs]="messages_colDefs" />
        <h2>Actions</h2>
        <ag-grid-angular style="width: 100%; height: 400px" [rowData]="actions" [columnDefs]="actions_colDefs" />
      </div>
      </mat-dialog-content>
      <mat-dialog-actions align="end">
        <button matButton mat-dialog-close>Close</button>
      </mat-dialog-actions>
  `,
  imports: [
    AgGridAngular,
    MatDialogModule,
    MatCardModule,
    MatButtonModule,
  ]
})
export class MatchDetailsComponent {
  protected data = inject<MatchItem>(MAT_DIALOG_DATA);

  protected messages = this.data.messages;
  protected actions = this.data.actions;

  protected messages_colDefs: ColDef<MessageInfo>[] = [
    { flex: 0.2, field: 'tick', valueFormatter: params => ticks2time(params.data?.tick ?? 0) },
    { flex: 0.2, field: 'sender' },
    { flex: 0.2, field: 'receiver' },
    { flex: 1, field: 'body' }
  ];

  protected actions_colDefs: ColDef<ActionInfo>[] = [
    { flex: 1, field: 'relic_id', headerName: 'Relic ID' },
    { flex: 1, field: 'name', filter: 'agTextColumnFilter' },
    { flex: 1, field: 'tick', valueFormatter: params => ticks2time(params.data?.tick ?? 0) },
    {
      flex: 1, field: 'data',
      headerName: 'Type',
      valueFormatter: params => getActionType(params.data?.data)
    },
    {
      flex: 1, field: 'data',
      headerName: 'Player location ID / SIM ID',
      valueFormatter: params => params.data?.data[1].toString() ?? ''
    },
    {
      flex: 1, field: 'data',
      headerName: 'Action counter(s)',
      valueFormatter: params => `${params.data?.data[4]?.toString()} | ${params.data?.data[5]?.toString()}`
    },
    {
      flex: 1, field: 'data',
      headerName: 'Action Context',
      valueFormatter: params => getActionContext([params.data?.data[10], params.data?.data[11]])
    },
    {
      flex: 1, field: 'data',
      headerName: 'Item ID',
      valueFormatter: params => params.data?.data[12]?.toString() ?? ''
    },
    { flex: 1, field: 'data' }
  ];
}

function getActionContext(data: [number | undefined, number | undefined]): string {
  return `${data[0]} | ${data[1]}`;
}

function getActionType(data: number[] | undefined): string {
  if (!data) { return '' };

  switch (data[0]) {
    case 3: return 'Build unit';
    case 5: return 'Cancel unit or wargear';
    case 15: return 'Upgrade Building';
    case 47: return 'Capture Point';
    case 49: return 'Reinforce unit';
    case 50: return 'Purchase wargear';
    case 51: return 'Cancel wargear purchase';
    case 78: return 'Place building';
    case 85: return 'Global ability';
    default:
      return 'unknown';
  }
}
