import { computed, Injectable, signal } from '@angular/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { load, Store } from '@tauri-apps/plugin-store';
import { MatchItem, ReplayInfo } from 'src/types';

const appWindow = getCurrentWebviewWindow()

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  private matches_state = signal<MatchItem[]>([]);

  private knownMatchIDs = new Set<number>();

  private json_store = signal<Store | undefined>(undefined);

  public matchList = computed(() => this.matches_state());

  constructor() {
    appWindow.listen<string>('new-game', (event) => {
      let json: ReplayInfo = JSON.parse(event.payload);

      // Do not add existing matches multiple times
      if (this.knownMatchIDs.has(json.id)) {
        return;
      }

      this.matches_state.update(state => {
        let match = mapJsonToVM(json);
        this.knownMatchIDs.add(match.match_id);

        // write matches to json store for permanent storage
        this.json_store()?.set('matches', [match, ...state]);

        // write matches to local state for ui
        return [match, ...state]
      });
    });

    load('store.json')
      .then(store => {
        this.json_store.set(store);
        this.json_store()?.get<MatchItem[]>('matches').then(matches => this.matches_state.set(matches ?? []));
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
