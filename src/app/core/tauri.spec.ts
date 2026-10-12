import { TestBed } from '@angular/core/testing';
import { Tauri } from './tauri';

describe('Tauri', () => {
  let service: Tauri;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Tauri);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
