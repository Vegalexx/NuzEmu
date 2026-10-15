import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Emulator } from './emulator';

describe('Emulator', () => {
  let component: Emulator;
  let fixture: ComponentFixture<Emulator>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Emulator],
    }).compileComponents();

    fixture = TestBed.createComponent(Emulator);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
