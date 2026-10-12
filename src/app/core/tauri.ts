import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

@Injectable({
    providedIn: 'root',
})
export class TauriService {
    greet(name: string): Promise<string> {
        return invoke<string>('greet', { name });
    }
}
