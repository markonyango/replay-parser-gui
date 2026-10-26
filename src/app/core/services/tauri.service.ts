import { computed, inject, Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { load, Store } from '@tauri-apps/plugin-store';
import { MatchItem, ReplayInfo } from 'src/types';
import { migrateStoredMatch, StoredMatch } from '../upload-status';
import { StatusService, StatusSnapshot } from './status.service';

const appWindow = getCurrentWebviewWindow();

@Injectable({
    providedIn: 'root',
})
export class TauriService {
    private status = inject(StatusService);

    private matches_state = signal<MatchItem[]>([]);

    private knownMatchIDs = new Set<number>();

    private json_store = signal<Store | undefined>(undefined);

    public matchList = computed(() => this.matches_state());

    // Resolves (never rejects) once the persisted matches have been loaded.
    private storeReady: Promise<void>;

    constructor() {
        this.storeReady = this.loadStore();

        // Events wait for the store so they are neither lost nor overwritten by the initial load.
        appWindow
            .listen<ReplayInfo>('new-game', (event) => {
                this.storeReady.then(() => this.addMatch(event.payload));
            })
            .catch((error) =>
                console.error('Could not listen for new games', error)
            );

        appWindow
            .listen<StatusSnapshot>('status-changed', (event) =>
                this.status.apply(event.payload)
            )
            // The backend may have reported status before this listener existed.
            .then(() => invoke<StatusSnapshot>('get_status'))
            .then((snapshot) => this.status.apply(snapshot))
            .catch((error) => console.error('Could not load status', error));
    }

    private async loadStore() {
        try {
            const store = await load('store.json');
            this.json_store.set(store);

            const stored = await store.get<StoredMatch[]>('matches');
            if (stored) {
                const matches = stored.map(migrateStoredMatch);
                this.matches_state.set(matches);
                matches.forEach((match) =>
                    this.knownMatchIDs.add(match.match_id)
                );

                // Persist migrations once so old entries stop carrying legacy data.
                if (matches.some((match, i) => match !== stored[i])) {
                    await store.set('matches', matches);
                }
            }
        } catch (error) {
            console.error('Could not load stored matches', error);
        }
    }

    private async addMatch(json: ReplayInfo) {
        // Do not add existing matches multiple times
        if (this.knownMatchIDs.has(json.id)) {
            return;
        }

        const match = mapJsonToVM(json);
        this.knownMatchIDs.add(match.match_id);

        const matches = [match, ...this.matches_state()];

        // write matches to local state for ui
        this.matches_state.set(matches);

        // write matches to json store for permanent storage
        try {
            await this.json_store()?.set('matches', matches);
        } catch (error) {
            console.error('Could not persist match', error);
        }
    }

    delete_match(id: number) {
        const filteredMatches = this.matches_state().filter(
            (match) => match.match_id !== id
        );

        // write data to the ui state only when updating the state on disk was successful
        this.json_store()
            ?.set('matches', filteredMatches)
            .then(() => {
                this.matches_state.set(filteredMatches);
                this.knownMatchIDs.delete(id);
            })
            .catch((error) => console.error('Could not delete match', error));
    }
}

function mapJsonToVM(json: ReplayInfo): MatchItem {
    return {
        match_id: json.id,
        players: json.players,
        map: json.map,
        duration: ticks2time(json.ticks),
        status: json.status,
        played_at: json.date,
        messages: json.messages,
        actions: json.actions,
    };
}

export function ticks2time(ticks: number) {
    const total_seconds = Math.floor(ticks / 10);
    const minutes = Math.floor(total_seconds / 60);
    const remaining_seconds = total_seconds - minutes * 60;

    return `${minutes < 10 ? '0' + minutes : minutes}:${
        remaining_seconds < 10 ? '0' + remaining_seconds : remaining_seconds
    }`;
}
