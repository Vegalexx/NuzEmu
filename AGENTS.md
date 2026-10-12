# AGENTS.md

## Projet

NuzEmu est un projet personnel open source : une application de bureau destinée à accompagner les joueurs Pokémon, notamment en Nuzlocke.

Objectifs à terme :
- Suivi des règles, rencontres, captures, pertes et progression Nuzlocke.
- Consultation de l'équipe et du PC du jeu.
- Teambuilder basé sur les Pokémon réellement présents dans la partie.
- Calculateur de dégâts.
- Informations sur les dresseurs.
- Communication avec un émulateur pour lire les données de la partie.

Priorités de plateformes :
1. Windows desktop.
2. Jeux Pokémon GBA comme preuve de concept.
3. Évaluer ensuite DS et 3DS.
4. Mobile et autres plateformes beaucoup plus tard.

Ne pas construire toutes les fonctionnalités en même temps. Valider les fondations avec des prototypes courts.

## Environnement vérifié

- Windows 11, build 10.0.26200 affiché par les outils.
- Visual Studio Code.
- Git Bash / MINGW64.
- Dossier : `C:\Users\vegal\Workspace\NuzEmu`.
- Node.js 24.21.0.
- npm 11.19.0.
- Angular CLI 22.1.8.
- Rust 1.99.0.
- Cargo 1.99.0.
- Toolchain Rust : `stable-x86_64-pc-windows-msvc`.
- Visual Studio Build Tools 2022 avec MSVC.
- WebView2 installé.
- Tauri CLI 2.12.2 et Tauri Rust 2.12.3.

Vérifier les versions réellement installées si l'environnement a évolué.

## Stack et architecture souhaitée

- Angular standalone, TypeScript et SCSS pour l'interface.
- Tauri 2 pour l'application de bureau.
- Rust pour les commandes natives et la logique système.
- npm pour les dépendances JavaScript.
- SQLite est envisagé pour le stockage local.

À terme, séparer :
- Interface Angular.
- Services frontend.
- Commandes Tauri.
- Logique métier indépendante de l'interface.
- Adaptateurs spécifiques aux émulateurs.
- Persistance locale.

Éviter de placer toute la logique dans le composant racine. Lire les fichiers existants avant de les remplacer.

## État actuel confirmé

- Le projet Angular a été créé à la racine.
- Tauri 2 est initialisé.
- `npm run tauri dev` démarre correctement une fenêtre Windows.
- `npm run build` a été utilisé pour vérifier la compilation Angular.
- Chemin des assets : `dist/nuzemu/browser`.
- Configuration Tauri : `frontendDist = ../dist/nuzemu/browser`.
- URL de développement : `http://localhost:4200/`.
- `@tauri-apps/api` a été installé.
- La commande Rust `greet` fonctionne depuis Angular.
- Le message affiché au test est `Bienvenue dans NuzEmu, Dresseur !`.

### Fichiers importants

- `src-tauri/src/lib.rs` : commande `greet`, enregistrement des commandes et plugin de logs.
- `src/app/app.ts` : logique du composant racine.
- `src/app/app.html` : template HTML externe.
- `src/app/app.scss` : styles du composant racine.
- `src/app/core/tauri.service.ts` : service frontend utilisant `invoke` depuis `@tauri-apps/api/core`.

Le nom exact de la classe Angular dépend du code présent. Le consulter avant toute modification.

### Angular signals

Les propriétés `message`, `error` et `loading` sont des signals. Dans le template, les lire avec des parenthèses :

- `message()`
- `error()`
- `loading()`

Le composant utilise `templateUrl: './app.html'` et `styleUrl: './app.scss'`.

Ne pas remplacer aveuglément `lib.rs` : conserver le `setup` et le plugin `tauri-plugin-log` déjà présents.

## Commandes utiles

Depuis la racine du projet :

```bash
npx tauri info
npm run build
npm run tauri dev
git status