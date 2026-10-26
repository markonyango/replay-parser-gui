import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlayerInfo } from 'src/types';
import { PlayersCellComponent } from './players-cell.component';

const TROPHY = '\u{1F3C6}';
const SKULL = '\u{2620}';

function player(name: string, team: number, status: PlayerInfo['status']): PlayerInfo {
  return { name, team, status } as PlayerInfo;
}

describe('PlayersCellComponent', () => {
  let component: PlayersCellComponent;
  let fixture: ComponentFixture<PlayersCellComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PlayersCellComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlayersCellComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('marks the team of the winning player', () => {
    component.players.set([player('Alice', 0, 'Killed'), player('Bob', 1, 'Won')]);
    fixture.detectChanges();

    const icons = [...fixture.nativeElement.querySelectorAll('[data-result]')]
      .map((el: HTMLElement) => el.textContent?.trim());
    expect(icons).toEqual([SKULL, TROPHY]);
  });

  it('shows no result when nobody won', () => {
    component.players.set([player('Alice', 0, 'Unknown'), player('Bob', 1, 'Playing')]);
    fixture.detectChanges();

    expect(component.winner()).toBeNull();
    expect(fixture.nativeElement.querySelectorAll('[data-result]').length).toBe(0);
  });
});
