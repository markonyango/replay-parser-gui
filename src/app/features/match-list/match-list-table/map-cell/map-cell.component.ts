import { Component, computed, signal } from '@angular/core';
import { ICellRendererAngularComp } from 'ag-grid-angular';
import { ICellRendererParams } from 'ag-grid-community';
import { MapInfo } from 'src/types';

@Component({
  selector: 'app-map-cell',
  templateUrl: './map-cell.component.html',
  styleUrls: ['./map-cell.component.css'],
  standalone: true
})
export class MapCellComponent implements ICellRendererAngularComp {
  agInit(params: ICellRendererParams<any, any, any>): void {
    this.map.set(params.value);
  }
  refresh(params: ICellRendererParams<any, any, any>): boolean {
    return false;
  }

  map = signal<MapInfo | undefined>(undefined);

  mapname = computed(() => this.map()?.path.replace('DATA:maps\\pvp\\', ''));
}
