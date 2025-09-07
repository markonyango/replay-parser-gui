import { Component, computed, inject, input, signal } from '@angular/core';
import { MatDialogModule, MatDialog } from '@angular/material/dialog';
import { AgGridAngular } from 'ag-grid-angular';
import { AllCommunityModule, ClientSideRowModelModule, GridApi, GridOptions, ModuleRegistry, themeMaterial } from 'ag-grid-community';

import { ReplayInfo } from 'src/types';
import { MapCellComponent } from './map-cell/map-cell.component';
import { PlayersCellComponent } from './players-cell/players-cell.component';
import { StatusCellComponent } from './status-cell/status-cell.component';
import { MatchDetailsComponent } from './match-details.component';
import { DetailsCellComponent } from './details-cell/details-cell.component';

ModuleRegistry.registerModules([AllCommunityModule, ClientSideRowModelModule]);

const theme = themeMaterial
  .withParams({
    browserColorScheme: "light",
    headerFontSize: 14
  });

@Component({
  selector: 'app-match-list-table',
  templateUrl: './match-list-table.component.html',
  styleUrls: ['./match-list-table.component.css'],
  imports: [AgGridAngular, MatDialogModule],
  standalone: true
})
export class MatchListTableComponent {
  dataSource = input<Partial<ReplayInfo>[]>([]);

  private _gridApi = signal<GridApi | undefined>(undefined);
  private _matDialog = inject(MatDialog);

  gridOptions = computed<GridOptions>(() => ({
    theme,
    onGridReady: event => {
      this._gridApi.set(event.api);
    },
    cellSelection: false,
    suppressCellFocus: true,
    columnDefs: [
      { flex: 1, field: 'match_id', headerName: 'Match ID' },
      { flex: 3, field: 'players', cellRenderer: PlayersCellComponent },
      { flex: 2, field: 'map', cellRenderer: MapCellComponent },
      { flex: 1, field: 'duration' },
      { flex: 1, field: 'status', headerName: 'Uploaded', cellRenderer: StatusCellComponent },
      { flex: 1, field: 'played_at', headerName: 'Played at' },
      { flex: 1, cellRenderer: DetailsCellComponent }
    ],
    onRowDoubleClicked: event => this._matDialog.open(MatchDetailsComponent, {
      hasBackdrop: true, data: event.data, height: '80vh', minWidth: '80vw'
    }),
  }));
}
