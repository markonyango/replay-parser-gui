import { Component, Input, OnChanges, SimpleChanges } from '@angular/core';
import { MapInfo } from 'src/types';

@Component({
    selector: 'app-map-cell',
    templateUrl: './map-cell.component.html',
    styleUrls: ['./map-cell.component.css'],
    standalone: false
})
export class MapCellComponent implements OnChanges {
  @Input() map: MapInfo | undefined;

  mapname: string | undefined;

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    this.mapname = this.map?.path.replace('DATA:maps\\pvp\\', '');
  }
}
