import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface PokemonInParty {
    slot: number;
    name: string;
    speciesInternal: number;
    nationalDex: number | null;
    level: number;
    hp: number;
    maxHp: number;
    checksumValid: boolean;
}

export interface PartySnapshot {
    gameTitle: string;
    gameCode: string;
    partyCount: number;
    party: PokemonInParty[];
}

export interface LibretroVideoFrame {
    width: number;
    height: number;
    frameId: number;
    rgbaBase64: string;
}

@Injectable({
    providedIn: 'root',
})
export class TauriService {
    greet(name: string): Promise<string> {
        return invoke<string>('greet', { name });
    }

    diagnosticNuzEmu(): Promise<string> {
        return invoke<string>('diagnostic_nuzemu');
    }

    checkLibretroCore(): Promise<string> {
        return invoke<string>('check_libretro_core');
    }

    bootLibretroRom(): Promise<string> {
        return invoke<string>('boot_libretro_rom');
    }

    runLibretroFrame(): Promise<string> {
        return invoke<string>('run_libretro_frame');
    }

    getLibretroFrame(): Promise<LibretroVideoFrame> {
        return invoke<LibretroVideoFrame>('get_libretro_frame');
    }
    runLibretroFrames(frames: number): Promise<string> {
        return invoke<string>('run_libretro_frames', { frames });
    }

    resetLibretroRom(): Promise<string> {
        return invoke<string>('reset_libretro_rom');
    }

    setLibretroButton(
        button: string,
        pressed: boolean,
    ): Promise<void> {
        return invoke<void>('set_libretro_button', {
            button,
            pressed,
        });
    }

    async getPartySnapshot(): Promise<PartySnapshot | null> {
        const raw = await invoke<string>('get_party_snapshot');

        if (!raw) {
            return null;
        }

        const data: unknown = JSON.parse(raw);

        if (
            typeof data !== 'object' ||
            data === null ||
            !('gameCode' in data) ||
            !('party' in data) ||
            !Array.isArray(data.party)
        ) {
            return null;
        }

        const snapshot = data as PartySnapshot;

        if (
            snapshot.gameCode !== 'AGB-BPEF' ||
            !snapshot.party.every(
                (pokemon) =>
                    typeof pokemon === 'object' &&
                    pokemon !== null &&
                    typeof pokemon.name === 'string' &&
                    typeof pokemon.level === 'number' &&
                    typeof pokemon.hp === 'number' &&
                    typeof pokemon.maxHp === 'number',
            )
        ) {
            return null;
        }

        return snapshot;
    }
}
