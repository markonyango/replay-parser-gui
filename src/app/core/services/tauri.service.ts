import { computed, Injectable, signal } from '@angular/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { MatchItem, ReplayInfo } from 'src/types';

const appWindow = getCurrentWebviewWindow()

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  private state = signal<MatchItem[]>([]);

  private knownMatchIDs = new Set<number>();

  public matchList = computed(() => this.state());

  constructor() {
    appWindow.listen<string>('new-game', (event) => {
      console.log(event.payload);
      let json: ReplayInfo = JSON.parse(event.payload);
      console.log(json);

      // Do not add existing matches multiple times
      if (this.knownMatchIDs.has(json.id)) {
        return;
      }

      this.state.update(state => {
        let match = mapJsonToVM(json);
        this.knownMatchIDs.add(match.match_id);
        return [...state, match]
      });
    });
  }
}

function mapJsonToVM(json: ReplayInfo): MatchItem {
  let status;
  try {
    status = JSON.parse(json.status);
  } catch (error) {
    status = { error: json.status }
  }

  return {
    match_id: json.id,
    players: json.players,
    map: json.map,
    duration: ticks2time(json.ticks),
    status,
    played_at: json.date,
    messages: json.messages,
    actions: json.actions
  };

}

export function ticks2time(ticks: number) {
  const total_seconds = Math.floor(ticks / 10);
  const minutes = Math.floor(total_seconds / 60);
  const remaining_seconds = total_seconds - minutes * 60;

  return `${minutes < 10 ? '0' + minutes : minutes}:${remaining_seconds < 10 ? '0' + remaining_seconds : remaining_seconds
    }`;
}
