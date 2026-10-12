
import { Component, inject, signal } from '@angular/core';
import { TauriService } from './core/tauri';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  private readonly tauri = inject(TauriService);

  readonly message = signal('');
  readonly error = signal('');
  readonly loading = signal(false);

  async testRust(): Promise<void> {
    this.message.set('Clic détecté, appel de Rust...');
    this.error.set('');
    this.loading.set(true);

    try {
      const result = await this.tauri.greet('Dresseur');
      this.message.set(result);
    } catch (err) {
      console.error('Échec de la commande Rust :', err);
      this.error.set(String(err));
    } finally {
      this.loading.set(false);
    }
  }
}
