import { signal } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TauriService } from '../../core/services/tauri.service';

import { MatchListComponent } from './match-list.component';

describe('MatchListComponent', () => {
  let component: MatchListComponent;
  let fixture: ComponentFixture<MatchListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MatchListComponent],
      providers: [
        {
          provide: TauriService,
          useValue: { matchList: signal([]), delete_match: () => {} },
        },
      ],
    })
    .compileComponents();

    fixture = TestBed.createComponent(MatchListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
