import { TestBed } from '@angular/core/testing';
import { emit } from '@tauri-apps/api/event';
import { ReplayInfo } from 'src/types';

import { TauriService } from './tauri.service';

describe('TauriService', () => {
  let service: TauriService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(TauriService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  it('adds a match from a new-game event carrying an object payload', async () => {
    // Let the constructor's listener registration settle before emitting.
    await new Promise((resolve) => setTimeout(resolve));

    await emit('new-game', {
      id: 42,
      ticks: 650,
      date: '2026-10-26',
      status: { kind: 'pending' },
      players: [],
      map: { name: 'Calderis', path: 'DATA:maps\\pvp\\6p_calderis' },
      messages: [],
      actions: [{ relic_id: 1, name: 'Alice', tick: 10, command: null, command_error: null }],
    } as unknown as ReplayInfo);
    // The handler waits for the store load before adding the match.
    await new Promise((resolve) => setTimeout(resolve, 50));

    const [match] = service.matchList();
    expect(match.match_id).toBe(42);
    expect(match.duration).toBe('01:05');
    expect(match.actions.length).toBe(1);
  });
});
