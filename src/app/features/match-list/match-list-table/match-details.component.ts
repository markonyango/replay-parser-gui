import { JsonPipe } from '@angular/common';
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
      <div class="flex flex-col gap-4">
        <h2>Messages</h2>
        <ag-grid-angular style="width: 100%; height: 300px" [rowData]="messages" [columnDefs]="messages_colDefs" />
        <h2>Actions</h2>
        <ag-grid-angular style="width: 100%; height: 300px" [rowData]="actions" [columnDefs]="actions_colDefs" />
      </div>
        <pre><code>{{ data | json }}</code></pre>
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
    JsonPipe
  ]
})
export class MatchDetailsComponent {
  protected data = inject<MatchItem>(MAT_DIALOG_DATA);

  protected messages = this.data.messages;
  protected actions = this.data.actions;

  protected messages_colDefs: ColDef<MessageInfo>[] = [
    { flex: 1, field: 'tick', valueFormatter: params => ticks2time(params.data?.tick ?? 0) },
    { flex: 1, field: 'sender' },
    { flex: 1, field: 'body' }
  ];

  protected actions_colDefs: ColDef<ActionInfo>[] = [
    { flex: 1, field: 'relic_id' },
    { flex: 1, field: 'name' },
    { flex: 1, field: 'tick', valueFormatter: params => ticks2time(params.data?.tick ?? 0) },
    {
      flex: 1, field: 'data',
      headerName: 'Type',
      valueFormatter: params => getActionType(params.data?.data[0])
    },
    {
      flex: 1, field: 'data',
      headerName: 'Action Context',
      valueFormatter: params => getActionContext([params.data?.data[10], params.data?.data[11]])
    },
    {
      flex: 1, field: 'data',
      headerName: 'Item ID',
      valueFormatter: params => params.data?.data[12].toString() ?? ''
    },
    { flex: 1, field: 'data' }
  ];
}

function getActionContext(data: [number | undefined, number | undefined]): string {
  return `${data[0]} | ${data[1]}`;
}

function getActionType(data: number | undefined): string {
  switch (data) {
    case 3: return 'Build unit';
    case 5: return 'Cancel unit or wargear';
    case 15: return 'Upgrade Building';
    case 47: return 'Capture Point';
    case 49: return 'Reinforce unit';
    case 50: return 'Purchase wargear';
    case 51: return 'Cancel wargear purchase';
    case 78: return 'Place building';
    default:
      return 'unknown';
  }
}
// - 1 => Ability on placeable object
// - 3 => Build unit
// - 5 => Cancel unit or wargear
// - 9 => Unknown // source: 0x10
// - 11 => Set rally point
// - 15 => Upgrade building
// - 23 => Exit building
// - 43 => Stop move
// - 44 => Move
// - 47 => Capture point
// - 48 => Attack
// - 49 => Reinforce unit
// - 50 => Purchase wargear
// - 51 => Cancel wargear purchase
// - 52 => Attack move
// - 53 => Ability on unit
// - 56 => Enter building or vehicle
// - 58 => Exit vehicle
// - 61 => Retreat
// - 70 => Force melee
// - 71 => Toggle stance
// - 78 => Place building
// - 85 => Global ability
// - 89 => Unknown
// - 94 => Unknown // source 0x0
// - 96 => Unknown // source 0x0
// - 98 => Unknown // source 0x0
