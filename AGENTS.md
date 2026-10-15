# AGENTS.md

## Projet

NuzEmu est un projet personnel open source : une application de bureau destinée à accompagner les joueurs Pokémon, notamment en Nuzlocke.

Objectifs à terme :
- Suivi des règles, rencontres, captures, pertes et progression Nuzlocke.
- Consultation de l'équipe et du PC du jeu.
- Teambuilder basé sur les Pokémon réellement présents dans la partie.
- Calculateur de dégâts.
- Informations sur les dresseurs.
- Émulation du jeu directement dans NuzEmu.
- Lecture des données de la partie pour alimenter les outils d'accompagnement.

### Priorités

1. Windows desktop.
2. Pokémon GBA comme première plateforme de validation.
3. Intégration d'un moteur d'émulation natif dans NuzEmu.
4. Évaluer ensuite les générations DS et 3DS.
5. Mobile et autres plateformes beaucoup plus tard.

Ne pas construire toutes les fonctionnalités en même temps. Valider les fondations par étapes courtes, testables et réversibles.

## Environnement de développement

Environnement vérifié :
- Windows 11.
- Visual Studio Code.
- Git Bash / MINGW64.
- Dossier du projet : `C:\Users\vegal\Workspace\NuzEmu`.
- Node.js 24.21.0.
- npm 11.19.0.
- Angular CLI 22.1.8.
- Rust 1.99.0.
- Cargo 1.99.0.
- Toolchain Rust : `stable-x86_64-pc-windows-msvc`.
- Visual Studio Build Tools 2022 avec MSVC.
- WebView2 installé.
- Tauri CLI 2.12.2.
- Tauri Rust 2.12.3.
- `@tauri-apps/api` version majeure 2.

Vérifier les versions réellement installées si l'environnement a évolué.

## Stack et architecture

- Angular standalone, TypeScript et SCSS pour l'interface.
- Tauri 2 pour l'application de bureau.
- Rust pour les commandes natives et la logique système.
- npm pour les dépendances JavaScript.
- SQLite envisagé pour la persistance locale, pas encore implémenté.

Séparation souhaitée :
- `src/app/features/` : composants fonctionnels de l'interface.
- `src/app/core/` : services et intégrations frontend partagés.
- `src-tauri/src/` : logique native Rust et commandes Tauri.
- Modules métier indépendants de l'interface.
- Adaptateurs spécifiques aux moteurs et formats de sauvegarde.
- Couche de persistance locale.

Éviter de concentrer la logique dans le composant racine.

## Règles de développement

### Avant de modifier le code

- Lire les fichiers existants et leurs imports avant de les remplacer.
- Vérifier les noms réels des fichiers, classes et fonctions.
- Privilégier des modifications ciblées plutôt que de réécrire des fichiers entiers.
- Préserver les fonctionnalités existantes sauf décision explicite.
- Ne pas supposer qu'un fichier généré est inutile sans vérifier son rôle.
- Donner les chemins exacts des fichiers et des commandes à exécuter.
- Distinguer clairement les fonctionnalités opérationnelles, les prototypes et les objectifs futurs.

### Angular

- Utiliser les composants standalone.
- Séparer le template HTML, les styles SCSS et la logique TypeScript.
- Conserver les conventions de nommage déjà établies par le projet.
- Les fichiers des composants utilisent actuellement les noms `*.ts`, `*.html` et `*.scss` propres à leur fonctionnalité.
- Utiliser les signals Angular lorsque cela simplifie l'état réactif.
- Dans les templates Angular, lire un signal avec `signalName()`.
- Déclarer les composants standalone utilisés dans `imports` du décorateur `@Component`.
- Vérifier que les imports pointent vers la bonne classe exportée.
- Ne pas ajouter de dépendance sans besoin concret.

Le service Tauri frontend est situé ici :

`src/app/core/tauri.ts`

Ne pas le renommer arbitrairement en `tauri.service.ts`.

### Interface et styles

L'interface est pensée pour une fenêtre desktop sombre.

Principes actuels :
- Panneau latéral gauche pour l'équipe Pokémon.
- Zone centrale pour le jeu.
- Barre d'outils supérieure et barre d'informations inférieure.
- Suppression des marges par défaut du document.
- Le document et l'application doivent occuper toute la fenêtre.
- Les styles globaux se trouvent dans `src/styles.scss`.
- Les styles du composant racine se trouvent dans `src/app/app.scss`.
- Les styles spécifiques restent dans les fichiers SCSS des composants concernés.

Ne pas supprimer les règles de grille ou les sélecteurs utilisés dans `app.html` lors d'une modification CSS.

### Tauri et Rust

- Utiliser Tauri 2.
- Conserver les commandes Tauri existantes lorsqu'elles sont encore utiles.
- Examiner `src-tauri/src/lib.rs` avant toute modification.
- Préserver le `setup`, le plugin de logs et l'enregistrement des commandes existantes, sauf refonte explicitement décidée.
- Ne pas exposer de commande Rust au frontend sans l'enregistrer dans `invoke_handler`.
- Traiter les erreurs de communication et de lecture des données explicitement.

### Émulation

L'objectif est d'intégrer l'émulation dans la fenêtre NuzEmu, sans demander à l'utilisateur de lancer un émulateur externe.

Piste technique étudiée :
- Utiliser le moteur mGBA, de préférence via son cœur Libretro si cette solution reste adaptée.
- Charger et piloter le cœur depuis le backend natif.
- Gérer le chargement de ROM, les entrées clavier/manette, le rendu vidéo, l'audio et les sauvegardes.
- Alimenter ensuite les fonctionnalités Nuzlocke avec les données du jeu.

Le cœur Libretro n'est pas encore intégré. Ne pas présenter l'écran de remplacement comme un émulateur opérationnel.

Ne pas faire transiter chaque image à 60 FPS par des appels Tauri `invoke` ordinaires sans avoir validé les performances et l'architecture de rendu.

Toute intégration du moteur doit prendre en compte :
- La licence du moteur choisi.
- Les contraintes de compilation et de distribution Windows.
- La gestion des sauvegardes et des états.
- Le cycle de vie du moteur et la fermeture propre de l'application.
- La séparation entre le moteur d'émulation et l'interface Angular.

L'utilisateur doit fournir ses propres ROM et BIOS lorsque leur utilisation est nécessaire. Ne pas intégrer ni distribuer de ROM commerciales.

## État actuel confirmé

### Application desktop

- Le projet Angular est créé à la racine.
- Tauri 2 est initialisé.
- `npm run tauri dev` démarre correctement une fenêtre Windows.
- La compilation Angular produit les fichiers dans `dist/nuzemu/browser`.
- La configuration Tauri utilise `frontendDist = ../dist/nuzemu/browser`.
- L'URL de développement Angular est `http://localhost:4200/`.
- `@tauri-apps/api` est installé.
- La commande Rust `greet` a fonctionné depuis Angular.
- La fenêtre a été configurée pour disposer d'une taille de travail adaptée au desktop.
- Les marges par défaut du document ont été supprimées.
- Le fond de l'application et la disposition générale ont été corrigés.

### Interface actuelle

Arborescence fonctionnelle :

```text
src/app/
├── app.config.ts
├── app.routes.ts
├── app.ts
├── app.html
├── app.scss
├── core/
│   ├── tauri.ts
│   └── tauri.spec.ts
└── features/
    ├── emulator/
    │   ├── emulator.ts
    │   ├── emulator.html
    │   ├── emulator.scss
    │   └── emulator.spec.ts
    └── party/
        ├── party.ts
        ├── party.html
        ├── party.scss
        └── party.spec.ts
```

Le composant racine assemble les composants `PartyComponent` et `EmulatorComponent`.

Le panneau d'équipe et la zone d'émulation sont séparés en composants standalone.

L'écran de l'émulateur est actuellement un placeholder avec un canvas de 240 × 160 pixels. Le rendu réel d'un jeu n'est pas encore implémenté.

Les styles globaux sont dans `src/styles.scss`. La mise en page principale est définie dans `src/app/app.scss`.

### Service frontend

Fichier : `src/app/core/tauri.ts`

Le service `TauriService` utilise `invoke` depuis `@tauri-apps/api/core`.

Il contient notamment :
- L'appel à la commande `greet`.
- Les types utilisés pour représenter l'équipe Pokémon.
- La récupération et la validation d'un instantané d'équipe via `get_party_snapshot`.

La lecture de l'équipe dépend encore du prototype TCP décrit ci-dessous.

### Prototype de lecture de l'équipe via mGBA

Jeu de validation :
- Pokémon Émeraude, version française.
- Code de jeu : `AGB-BPEF`.
- ROM de 16 Mio.
- Un Gobou de niveau 5 a été identifié pendant les tests.

Scripts disponibles :

- `scripts/mgba/diagnostic.lua` : diagnostic du jeu et des domaines mémoire.
- `scripts/mgba/party_probe.lua` : exploration des données de l'équipe.
- `scripts/mgba/socket_test.lua` : validation de l'API socket de mGBA.
- `scripts/mgba/nuzemu_bridge.lua` : prototype de transmission de données à NuzEmu.

Le prototype utilise TCP sur `127.0.0.1:18765`.

Le backend Rust ouvre un récepteur TCP et conserve le dernier instantané reçu. La commande Tauri `get_party_snapshot` permet au frontend de récupérer cette chaîne.

La connexion et l'envoi TCP ont été testés avec succès.

Cette intégration est un prototype de développement, pas l'architecture définitive de l'émulateur embarqué. Conserver les scripts tant que leur utilité pour le diagnostic n'a pas disparu.

### Backend Rust

Fichiers principaux :
- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

`lib.rs` contient actuellement :
- La commande `greet`.
- La commande `get_party_snapshot`.
- Un état partagé pour le dernier instantané d'équipe.
- Le démarrage du récepteur TCP.
- Le plugin de logs en mode debug.
- L'enregistrement des commandes Tauri.

Le cycle de vie du récepteur TCP et son arrêt propre doivent être revus lors de la refonte de l'intégration émulateur.

## Tests et fichiers générés

Ne pas supprimer automatiquement les fichiers de tests :
- `src/app/app.spec.ts`
- `src/app/core/tauri.spec.ts`
- `src/app/features/emulator/emulator.spec.ts`
- `src/app/features/party/party.spec.ts`

Les adapter aux comportements actuels lorsque nécessaire.

Les dossiers `node_modules`, `dist`, `target`, `.angular` et `.git` ne sont pas à inclure dans les commits. Vérifier `.gitignore` avant de modifier les exclusions.

Conserver les fichiers générés de configuration et de schéma Tauri s'ils sont suivis par Git et nécessaires au projet.

## Prochaines étapes

1. Vérifier que l'état courant est propre et effectuer un commit de checkpoint.
2. Vérifier la compilation et l'état des tests existants.
3. Examiner les tests Angular et supprimer uniquement les tests obsolètes après vérification.
4. Finaliser les bases de l'interface de l'équipe et de l'écran de jeu.
5. Choisir et intégrer le moteur d'émulation natif.
6. Valider progressivement le rendu, les contrôles, l'audio et les sauvegardes.
7. Remplacer le pont TCP externe par une intégration adaptée au moteur embarqué.
8. Définir ensuite une architecture de persistance et les fonctionnalités Nuzlocke.

## Commandes utiles

Depuis la racine du projet :

```bash
npx tauri info
npm run build
npm run tauri dev
git status --short
git diff --check
```

Avant un commit :
- Vérifier le diff.
- Ne pas inclure les ROM, BIOS, sauvegardes personnelles ou secrets.
- Vérifier que les fichiers générés volumineux ne sont pas suivis accidentellement.
- Utiliser un message de commit descriptif.
