import { Component, computed, inject, input, output, signal } from '@angular/core';
import { MatDialog, MatDialogModule } from '@angular/material/dialog';
import { AgGridAngular } from 'ag-grid-angular';
import { AllCommunityModule, ClientSideRowModelModule, GridApi, GridOptions, ModuleRegistry, themeMaterial } from 'ag-grid-community';

import { MatchItem } from 'src/types';
import { MapCellComponent } from './map-cell/map-cell.component';
import { PlayersCellComponent } from './players-cell/players-cell.component';
import { DetailsRendererComponent } from './details.renderer';
import { ActionsCellRendererComponent } from './actions.renderer';
import { DeleteRendererComponent } from './delete.renderer';
import { MatchDetailsComponent } from './match-details.component';

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
  dataSource = input<MatchItem[]>([]);

  delete = output<number>();

  private _gridApi = signal<GridApi | undefined>(undefined);

  private _dialog = inject(MatDialog);

  gridOptions = computed<GridOptions<MatchItem>>(() => ({
    theme,
    onGridReady: event => {
      this._gridApi.set(event.api);
    },
    cellSelection: false,
    suppressCellFocus: true,
    columnDefs: [
      { flex: 1, field: 'match_id', headerName: 'Match ID', sort: 'desc' },
      { flex: 3, field: 'players', cellRenderer: PlayersCellComponent },
      { flex: 2, field: 'map', cellRenderer: MapCellComponent },
      { flex: 1, field: 'duration' },
      { flex: 1, field: 'status', headerName: 'Uploaded', valueGetter: params => params.data?.status['response'] ?? params.data?.status['error'] },
      { flex: 1, field: 'played_at', headerName: 'Played at' },
      {
        flex: 1, cellRenderer: ActionsCellRendererComponent, cellRendererParams: {
          components: [
            {
              component: DetailsRendererComponent, action: (params: any) =>
                this._dialog.open(MatchDetailsComponent, { hasBackdrop: true, data: params.data, height: '80vh', minWidth: '80vw' })
            },
            { component: DeleteRendererComponent, action: (params: any) => this.delete.emit(params.data.match_id) }
          ]
        }
      }
    ],
  }));
}
