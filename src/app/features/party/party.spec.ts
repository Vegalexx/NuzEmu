import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Party } from './party';

describe('Party', () => {
  let component: Party;
  let fixture: ComponentFixture<Party>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Party],
    }).compileComponents();

    fixture = TestBed.createComponent(Party);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
