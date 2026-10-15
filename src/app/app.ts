
import { Component } from '@angular/core';
import { EmulatorComponent } from './features/emulator/emulator';
import { PartyComponent } from './features/party/party';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [PartyComponent, EmulatorComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App { }
