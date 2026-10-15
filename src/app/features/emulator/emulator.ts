import {
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  ViewChild,
  inject,
  signal
} from '@angular/core';
import { TauriService } from '../../core/tauri';

@Component({
  selector: 'app-emulator',
  standalone: true,
  templateUrl: './emulator.html',
  styleUrl: './emulator.scss',
})
export class EmulatorComponent implements OnDestroy {
  private readonly tauri = inject(TauriService);

  @ViewChild('gameCanvas')
  private gameCanvas?: ElementRef<HTMLCanvasElement>;

  readonly diagnosticMessage = signal('Prêt.');
  readonly isRunning = signal(false);
  readonly speed = signal<1 | 2 | 4>(1);

  private romLoaded = false;
  private animationFrameId: number | null = null;
  private lastFrameTime = 0;
  private busy = false;
  private resetRequested = false;

  private readonly frameDuration = 1000 / 60;

  async startGame(): Promise<void> {
    if (this.isRunning() || this.busy || this.resetRequested) {
      return;
    }

    this.busy = true;

    try {
      if (!this.romLoaded) {
        this.diagnosticMessage.set(
          'Chargement de Pokémon Émeraude...'
        );

        const message = await this.tauri.bootLibretroRom();
        this.romLoaded = true;

        await this.refreshFrame();
        this.diagnosticMessage.set(message);
      }

      this.lastFrameTime = 0;
      this.isRunning.set(true);

      this.diagnosticMessage.set(
        `Jeu en cours.Vitesse x${this.speed()}.`
      );
    } catch (error) {
      this.isRunning.set(false);
      this.diagnosticMessage.set(`Erreur: ${String(error)} `);
    } finally {
      this.busy = false;
    }

    if (this.isRunning()) {
      this.animationFrameId = requestAnimationFrame(this.tick);
    }
  }

  pauseGame(): void {
    this.isRunning.set(false);

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }

    this.diagnosticMessage.set('Jeu en pause.');
  }

  toggleGame(): void {
    if (this.isRunning()) {
      this.pauseGame();
    } else {
      void this.startGame();
    }
  }

  cycleSpeed(): void {
    const nextSpeed = this.speed() === 1
      ? 2
      : this.speed() === 2
        ? 4
        : 1;

    this.speed.set(nextSpeed);

    this.diagnosticMessage.set(
      `Vitesse d'émulation : x${nextSpeed}.`
    );
  }

  async resetGame(): Promise<void> {
    if (this.resetRequested) return;

    this.resetRequested = true;

    const resumeAfterReset = this.isRunning();

    // Arrête de planifier de nouvelles frames.
    this.pauseGame();

    this.diagnosticMessage.set(
      'Attente de la fin de la frame en cours...'
    );

    // Attend la fin de l'appel Rust éventuellement en cours.
    while (this.busy) {
      await new Promise<void>(resolve => setTimeout(resolve, 10));
    }

    this.busy = true;
    let resetSucceeded = false;

    try {
      this.releaseAllButtons();

      this.diagnosticMessage.set(
        'Redémarrage de Pokémon Émeraude...'
      );

      const message = await this.tauri.resetLibretroRom();

      this.romLoaded = true;
      await this.refreshFrame();

      this.diagnosticMessage.set(message);
      resetSucceeded = true;
    } catch (error) {
      this.romLoaded = false;

      this.diagnosticMessage.set(
        `Erreur pendant le redémarrage : ${String(error)}`
      );
    } finally {
      this.busy = false;
      this.resetRequested = false;
    }

    if (resetSucceeded && resumeAfterReset) {
      this.lastFrameTime = 0;
      this.isRunning.set(true);

      this.diagnosticMessage.set(
        `Jeu relancé. Vitesse x${this.speed()}.`
      );

      this.animationFrameId = requestAnimationFrame(this.tick);
    } else if (resetSucceeded) {
      this.diagnosticMessage.set(
        'ROM réinitialisée. Appuie sur Démarrer pour jouer.'
      );
    }
  }

  private readonly tick = async (timestamp: number): Promise<void> => {
    this.animationFrameId = null;

    if (!this.isRunning()) return;

    if (this.busy || this.resetRequested) {
      if (this.isRunning() && !this.resetRequested) {
        this.animationFrameId = requestAnimationFrame(this.tick);
      }

      return;
    }

    if (this.lastFrameTime === 0) {
      this.lastFrameTime = timestamp;
    }

    // Rattrape les frames perdues si un appel Rust prend trop de temps.
    const elapsed = timestamp - this.lastFrameTime;

    const normalFramesElapsed = Math.max(
      1,
      Math.floor(elapsed / this.frameDuration)
    );

    const framesToRun = Math.min(
      120,
      normalFramesElapsed * this.speed()
    );

    this.lastFrameTime = timestamp;
    this.busy = true;

    try {
      await this.tauri.runLibretroFrames(framesToRun);
      await this.refreshFrame();
    } catch (error) {
      this.pauseGame();

      this.diagnosticMessage.set(
        `Erreur pendant l'émulation : ${String(error)}`
      );

      return;
    } finally {
      this.busy = false;
    }

    if (this.isRunning()) {
      this.animationFrameId = requestAnimationFrame(this.tick);
    }
  };

  private async refreshFrame(): Promise<void> {
    const frame = await this.tauri.getLibretroFrame();
    const canvas = this.gameCanvas?.nativeElement;

    if (!canvas) {
      throw new Error('Canvas vidéo introuvable.');
    }

    const context = canvas.getContext('2d');

    if (!context) {
      throw new Error('Impossible de créer le contexte 2D.');
    }

    const binary = atob(frame.rgbaBase64);
    const pixels = new Uint8ClampedArray(binary.length);

    for (let i = 0; i < binary.length; i++) {
      pixels[i] = binary.charCodeAt(i);
    }

    if (
      canvas.width !== frame.width ||
      canvas.height !== frame.height
    ) {
      canvas.width = frame.width;
      canvas.height = frame.height;
    }

    context.putImageData(
      new ImageData(pixels, frame.width, frame.height),
      0,
      0
    );
  }

  private readonly keyMap: Record<string, string> = {
    enter: 'start',
    delete: 'select',
    z: 'up',
    q: 'left',
    s: 'down',
    d: 'right',
    a: 'l',
    e: 'r',
    l: 'b',
    m: 'a',
  };

  private readonly pressedKeys = new Set<string>();

  private isTypingTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
      return false;
    }

    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target.isContentEditable
    );
  }

  @HostListener('document:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent): void {
    if (this.isTypingTarget(event.target)) return;

    const key = event.key.toLowerCase();

    // Raccourcis globaux de l'émulateur.
    switch (key) {
      case 'tab':
        event.preventDefault();

        if (!event.repeat) {
          this.cycleSpeed();
        }

        return;

      case 'p':
        event.preventDefault();

        if (!event.repeat) {
          this.toggleGame();
        }

        return;

      case 'r':
        event.preventDefault();

        if (!event.repeat) {
          void this.resetGame();
        }

        return;
    }

    // Touches de jeu Pokémon existantes.
    const button = this.keyMap[key];

    if (!button) return;

    event.preventDefault();

    if (event.repeat || this.pressedKeys.has(key)) return;

    this.pressedKeys.add(key);
    void this.sendButton(button, true);
  }

  @HostListener('document:keyup', ['$event'])
  onKeyUp(event: KeyboardEvent): void {
    const key = event.key.toLowerCase();
    const button = this.keyMap[key];

    if (!button) return;

    event.preventDefault();
    this.pressedKeys.delete(key);
    void this.sendButton(button, false);
  }

  @HostListener('window:blur')
  onWindowBlur(): void {
    this.releaseAllButtons();
  }

  private async sendButton(
    button: string,
    pressed: boolean
  ): Promise<void> {
    try {
      await this.tauri.setLibretroButton(button, pressed);
    } catch (error) {
      this.diagnosticMessage.set(
        `Erreur de commande : ${String(error)}`
      );
    }
  }

  private releaseAllButtons(): void {
    for (const key of this.pressedKeys) {
      const button = this.keyMap[key];

      if (button) {
        void this.sendButton(button, false);
      }
    }

    this.pressedKeys.clear();
  }

  ngOnDestroy(): void {
    this.releaseAllButtons();
    this.pauseGame();
  }
}