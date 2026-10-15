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

@Injectable({
    providedIn: 'root',
})
export class TauriService {
    greet(name: string): Promise<string> {
        return invoke<string>('greet', { name });
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
