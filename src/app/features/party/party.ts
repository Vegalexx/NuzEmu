
import {
  Component,
  OnDestroy,
  OnInit,
  inject,
  signal,
} from '@angular/core';

import {
  TauriService,
  type PartySnapshot,
} from '../../core/tauri';

@Component({
  selector: 'app-party',
  standalone: true,
  imports: [],
  templateUrl: './party.html',
  styleUrl: './party.scss',
})
export class PartyComponent implements OnInit, OnDestroy {
  private readonly tauri = inject(TauriService);

  private pollingId?: ReturnType<typeof setInterval>;
  private requestInProgress = false;

  readonly snapshot = signal<PartySnapshot | null>(null);
  readonly loading = signal(true);
  readonly error = signal('');

  async ngOnInit(): Promise<void> {
    await this.refresh();

    this.pollingId = setInterval(() => {
      void this.refresh();
    }, 1000);
  }

  ngOnDestroy(): void {
    if (this.pollingId !== undefined) {
      clearInterval(this.pollingId);
    }
  }

  async refresh(): Promise<void> {
    if (this.requestInProgress) {
      return;
    }

    this.requestInProgress = true;

    try {
      const snapshot = await this.tauri.getPartySnapshot();

      this.snapshot.set(snapshot);
      this.error.set('');
    } catch (err) {
      console.error('Impossible de récupérer l’équipe :', err);
      this.error.set('Connexion au jeu indisponible.');
    } finally {
      this.loading.set(false);
      this.requestInProgress = false;
    }
  }

  getHpPercentage(hp: number, maxHp: number): number {
    if (maxHp <= 0) {
      return 0;
    }

    return Math.max(0, Math.min(100, (hp / maxHp) * 100));
  }

  trackPokemon(_: number, pokemon: { slot: number }): number {
    return pokemon.slot;
  }
}
