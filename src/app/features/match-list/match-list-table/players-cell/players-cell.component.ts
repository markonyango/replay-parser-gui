import { Component, computed, input } from '@angular/core';
import { PlayerInfo } from 'src/types';

@Component({
  selector: 'app-players-cell',
  templateUrl: './players-cell.component.html',
  styleUrls: ['./players-cell.component.css'],
  standalone: false
})
export class PlayersCellComponent {
  players = input<PlayerInfo[]>([]);

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
