import { Component, computed, input } from '@angular/core';
import { MapInfo } from 'src/types';

@Component({
  selector: 'app-map-cell',
  templateUrl: './map-cell.component.html',
  styleUrls: ['./map-cell.component.css'],
  standalone: false
})
export class MapCellComponent {
  map = input<MapInfo | undefined>(undefined);

  mapname = computed(() => this.map()?.path.replace('DATA:maps\\pvp\\', ''));
}
