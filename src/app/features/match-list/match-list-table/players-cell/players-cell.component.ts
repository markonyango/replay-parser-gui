import { Component, Input, OnInit } from '@angular/core';
import { PlayerInfo } from 'src/types';

@Component({
  selector: 'app-players-cell',
  templateUrl: './players-cell.component.html',
  styleUrls: ['./players-cell.component.css'],
})
export class PlayersCellComponent {
  @Input() players: PlayerInfo[] = [];

  teamOne: string = '';
  teamTwo: string = '';

  winner: number = 0;

  constructor() {}

  ngOnChanges() {
    this.teamOne = this.players
      .filter((player) => player.team === 0)
      .map((player) => player.name)
      .join(', ');

    this.teamTwo = this.players
      .filter((player) => player.team === 1)
      .map((player) => player.name)
      .join(', ');

    this.winner =
      this.players.find((player) => player.status === 'Won')?.team ?? 0;
  }
}
