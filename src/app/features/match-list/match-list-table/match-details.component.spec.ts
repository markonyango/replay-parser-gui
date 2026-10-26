import { ComponentFixture, TestBed } from '@angular/core/testing';
import { MAT_DIALOG_DATA } from '@angular/material/dialog';
import { NoopAnimationsModule } from '@angular/platform-browser/animations';
import { MatchItem, PlayerInfo } from 'src/types';

import { MatchDetailsComponent } from './match-details.component';

function player(name: string, team: number, status: PlayerInfo['status']): PlayerInfo {
  return { name, team, status, slot: team } as PlayerInfo;
}

function matchItem(overrides: Partial<MatchItem> = {}): MatchItem {
  return {
    match_id: 42,
    players: [player('Alice', 0, 'Won'), player('Bob', 1, 'Killed')],
    map: { name: 'Quralon', path: 'DATA:maps\\pvp\\2p_quralon' } as MatchItem['map'],
    duration: '12:34',
    status: { kind: 'pending' },
    played_at: '2026-01-01',
    messages: [
      { tick: 100, sender: 'Alice', receiver: 'All', body: 'gl hf', player_id: 0 },
      { tick: 200, sender: 'Bob', receiver: 'All', body: 'you too', player_id: 1 },
    ],
    actions: [],
    ...overrides,
  };
}

describe('MatchDetailsComponent', () => {
  let fixture: ComponentFixture<MatchDetailsComponent>;

  async function create(data: MatchItem) {
    await TestBed.configureTestingModule({
      imports: [MatchDetailsComponent, NoopAnimationsModule],
      providers: [{ provide: MAT_DIALOG_DATA, useValue: data }],
    }).compileComponents();

    fixture = TestBed.createComponent(MatchDetailsComponent);
    fixture.detectChanges();
  }

  it('shows match context and tab counts', async () => {
    await create(matchItem());

    const text: string = fixture.nativeElement.textContent;
    expect(text).toContain('2p_quralon');
    expect(text).toContain('12:34');
    expect(text).toContain('Alice');
    expect(text).toContain('Bob');
    expect(text).toContain('Messages (2)');
    expect(text).toContain('Actions (0)');
  });

  it('marks the team of the winning player', async () => {
    await create(matchItem());

    const text: string = fixture.nativeElement.textContent;
    expect(text).toContain('Winners:');
    expect(text).toContain('Defeated:');
  });

  it('shows no result when nobody won', async () => {
    await create(matchItem({
      players: [player('Alice', 0, 'Unknown'), player('Bob', 1, 'Playing')],
    }));

    const text: string = fixture.nativeElement.textContent;
    expect(text).not.toContain('Winners:');
    expect(text).not.toContain('Defeated:');
    expect(text).not.toContain('\u{1F3C6}');
  });

  it('filters messages by search text', async () => {
    await create(matchItem());

    const input: HTMLInputElement = fixture.nativeElement.querySelector(
      'input[aria-label="Search messages"]',
    );
    input.value = 'gl hf';
    input.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    const rows = fixture.nativeElement.querySelectorAll('[role="log"] li');
    expect(rows.length).toBe(1);
    expect(rows[0].textContent).toContain('Alice');
  });

  it('shows an empty state when there are no messages', async () => {
    await create(matchItem({ messages: [] }));

    expect(fixture.nativeElement.textContent).toContain('No messages in this replay.');
  });

  it('opens on the actions tab when there are no messages', async () => {
    await create(matchItem({
      messages: [],
      actions: [{ relic_id: 1, name: 'Alice', tick: 10 }],
    }));

    expect(fixture.nativeElement.textContent).toContain('Actions (1)');
    expect(fixture.nativeElement.textContent).toContain('Export CSV');
  });

  it('filters actions by player', async () => {
    await create(matchItem({
      messages: [],
      actions: [
        { relic_id: 1, name: 'Alice', tick: 10 },
        { relic_id: 2, name: 'Bob', tick: 20 },
        { relic_id: 2, name: 'Bob', tick: 30 },
      ],
    }));

    const select: HTMLSelectElement = fixture.nativeElement.querySelector(
      'select[aria-label="Filter actions by player"]',
    );
    select.value = 'Bob';
    select.dispatchEvent(new Event('change'));
    fixture.detectChanges();

    expect(fixture.nativeElement.textContent).toContain('2 shown');
  });
});
