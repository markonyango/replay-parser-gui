import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MatchListTableComponent } from './match-list-table.component';

describe('MatchListTableComponent', () => {
  let component: MatchListTableComponent;
  let fixture: ComponentFixture<MatchListTableComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ MatchListTableComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MatchListTableComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
