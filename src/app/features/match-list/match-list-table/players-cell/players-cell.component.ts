import { Component, computed, signal } from '@angular/core';
import { ICellRendererAngularComp } from 'ag-grid-angular';
import { ICellRendererParams } from 'ag-grid-community';
import { PlayerInfo } from 'src/types';

@Component({
  selector: 'app-players-cell',
  templateUrl: './players-cell.component.html',
  styleUrls: ['./players-cell.component.css'],
  standalone: true
})
export class PlayersCellComponent implements ICellRendererAngularComp {
  agInit(params: ICellRendererParams<any, any, any>): void {
    this.players.set(params.value);
  }

  refresh(params: ICellRendererParams<any, any, any>): boolean {
    return false;
  }

  players = signal<PlayerInfo[]>([]);

  teamOne = computed(() => this.players()
    .filter((player) => player.team === 0)
    .map((player) => player.name)
    .join(', '));

  teamTwo = computed(() => this.players()
    .filter((player) => player.team === 1)
    .map((player) => player.name)
    .join(', '));

  winner = computed(() => this.players().find(player => player.status === 'Won')?.team ?? 0);
}
