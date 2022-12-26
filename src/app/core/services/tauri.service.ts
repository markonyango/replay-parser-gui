import { Injectable } from '@angular/core';
import { appWindow } from '@tauri-apps/api/window';
import {
  distinctUntilKeyChanged,
  scan,
  Subject,
} from 'rxjs';
import { map, shareReplay } from 'rxjs/operators';
import { ReplayInfo } from 'src/types';

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  public matchList$;
  private _replays$;
  private _match: Subject<ReplayInfo> = new Subject<ReplayInfo>();

  constructor() {
    this._replays$ = this._match.asObservable().pipe(
      distinctUntilKeyChanged('id'),
      scan((acc, value) => [...acc, value], [] as ReplayInfo[]),
      shareReplay({ bufferSize: 1, refCount: true })
    );

    this.matchList$ = this._replays$.pipe(
      map((replays) =>
        replays.map((replay) => {
          let status;
          try {
            status = JSON.parse(replay.status);
          } catch(error) {
            status = { error: replay.status }
          }

          return {
            match_id: replay.id,
            players: replay.players,
            map: replay.map,
            duration: ticks2time(replay.ticks),
            status,
            played_at: replay.date,
          };
        })
      )
    );

    appWindow.listen<string>('new-game', (event) => {
      let json: ReplayInfo = JSON.parse(event.payload);

      this._match.next(json);
    });
  }
}

function ticks2time(ticks: number) {
  const total_seconds = Math.floor(ticks / 10);
  const minutes = Math.floor(total_seconds / 60);
  const remaining_seconds = total_seconds - minutes * 60;

  return `${minutes < 10 ? '0' + minutes : minutes}:${remaining_seconds < 10 ? '0' + remaining_seconds : remaining_seconds
    }`;
}
