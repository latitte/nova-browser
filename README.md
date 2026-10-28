# NOVA Browser

Un premier navigateur desktop en français, avec une interface calme et des pages affichées par de vraies WebViews natives. **MVP en développement : la compilation est vérifiée, la navigation native reste à valider sur un poste compatible.** Voir [les résultats de validation](docs/VALIDATION.md).

## Ce qui est implémenté

- Adresses HTTP(S), recherches DuckDuckGo/Google/Bing/Brave, suggestions locales.
- Retour, suivant, rechargement, arrêt et zoom via les commandes natives.
- Onglets verticaux ou horizontaux, réorganisation des onglets verticaux par glisser-déposer, duplication, épinglage, restauration des onglets fermés et de la session.
- Favoris et historique recherchables dans SQLite, effacement de l’historique.
- Nouvelle page d’accueil, thèmes clair/sombre/système, cinq accents, paramètres enregistrés automatiquement.
- Popups HTTP(S) ouvertes dans de nouveaux onglets, fermeture des WebViews lorsque les onglets sont fermés.
- Icône originale, animations CSS légères et respect de `prefers-reduced-motion`.

Ces fonctions sont codées ; leur présence ne signifie pas que tous leurs comportements sont validés sur les trois systèmes. Les limites sont détaillées dans [l’architecture](docs/ARCHITECTURE.md). Aucun contrôle ne simule une navigation par iframe. `npm run dev` seul ouvre uniquement l’interface ; la navigation exige l’exécutable Tauri.

## Développement local

Prérequis : Node.js 24, Rust 1.99.1 (rustup lit `rust-toolchain.toml`), compilateur C/C++ et dépendances Tauri de votre plateforme.

### Debian 13 / Ubuntu 24.04+

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
npm ci
npm run tauri dev
```

### Windows 10/11

Installer les outils de compilation Visual Studio avec la charge « Développement Desktop en C++ », Rust pour la cible MSVC et Microsoft Edge WebView2 Runtime. Puis exécuter `npm ci` et `npm run tauri dev` dans le dépôt. Le moteur est WebView2 ; sa compatibilité n’a pas encore été testée ici.

### macOS

Installer les Command Line Tools de Xcode (`xcode-select --install`), Node et Rust. Puis `npm ci` et `npm run tauri dev`. Les paquets non signés peuvent nécessiter une autorisation explicite de macOS ; la signature et la notarisation ne sont pas configurées.

## Vérifications

```sh
npm run check
npm test
npm run test:ui
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Test natif de développement (serveur Vite actif, affichage graphique disponible) :

```sh
python3 scripts/smoke-server.py
# Dans un second terminal :
NOVA_SMOKE_URL=http://127.0.0.1:18765 npm run tauri dev
```

Le test ouvre une vraie WebView, attend un titre calculé en JavaScript, navigue vers une seconde page, revient en arrière, vérifie SQLite et le refus d’une commande privilégiée provenant d’un onglet distant, puis ferme l’onglet. Son échec dans le conteneur actuel est documenté. Ne pas désactiver la sandbox pour le faire passer.

## Production

Sur chaque système cible :

```sh
npm run tauri build -- --bundles deb       # Linux .deb
npm run tauri build -- --bundles appimage  # Linux AppImage, non validé ici
npm run tauri build -- --bundles nsis      # Windows .exe, sur Windows
npm run tauri build -- --bundles dmg       # macOS .dmg, sur macOS
```

Les sorties sont dans `src-tauri/target/release/bundle/`. Les installateurs Windows/macOS nécessitent leurs machines cibles ; aucune compilation croisée n’est revendiquée. Le workflow GitHub Actions prépare ces trois cibles mais n’a pas été exécuté depuis cette session. Aucun certificat de signature n’est fourni.

## Environnement cloud

Dans cette machine Debian sans droits administrateur :

```sh
bash scripts/setup-cloud.sh
. /workspace/tooling/activate-nova.sh
npm run dev
```

L’installation locale permet de compiler et tester les fonctions indépendantes de l’affichage. Elle ne rend pas le moteur desktop exécutable dans ce conteneur : chemins système WebKit et espaces de noms indisponibles. Le mode web n’est pas un substitut à la validation native. Les fichiers d’outillage restent dans `/workspace/tooling`, hors du dépôt. Utiliser le checkout existant ; chaque tâche cloud est déjà isolée, aucun worktree supplémentaire n’est nécessaire.

## Données et confidentialité

SQLite et les données WebView restent dans les répertoires applicatifs de l’utilisateur. Sur cette machine, les variables XDG les redirigent dans `/workspace/tooling/user-*`. Aucun compte ni service applicatif distant n’est requis. Les recherches et pages visitées contactent naturellement leurs services respectifs. La politique réseau cloud limite les destinations accessibles ; un accès GitHub fonctionnel ne prouve pas l’accès à tout Internet.
