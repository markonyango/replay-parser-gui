import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlayersCellComponent } from './players-cell.component';

describe('PlayersCellComponent', () => {
  let component: PlayersCellComponent;
  let fixture: ComponentFixture<PlayersCellComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ PlayersCellComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlayersCellComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
