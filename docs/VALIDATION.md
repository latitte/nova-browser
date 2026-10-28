# Validation du MVP — 28 octobre 2026

## Résultats obtenus dans cette instance

Machine Debian 13 x86_64, Node 24.19.0, Rust 1.99.1. Dépôt initialement vide, sans commit ; aucun fichier utilisateur préexistant modifié. Les nouveaux fichiers restent non commités.

| Vérification | Résultat |
| --- | --- |
| Installation npm verrouillée (`npm ci`) | Réussie |
| Réexécution du script cloud et extraction locale Debian | Réussie, sans installation système ni modification des lockfiles |
| Typage Svelte/TypeScript (`npm run check`) | 0 erreur, 0 avertissement |
| Résolution des adresses et rejet des protocoles dangereux (`npm test`) | 2 tests réussis |
| Interface compilée dans jsdom (`npm run test:ui`) | 1 scénario réussi : onglets, sauvegarde du thème, favoris vides, refus de navigation hors application |
| Tests Rust | 4 réussis : HTTP(S), URL refusées, identifiants des onglets, migrations SQLite |
| Build Vite | Réussi : JavaScript 92,30 ko, 32,54 ko gzip ; CSS 14,80 ko, 3,66 ko gzip |
| Vérification et compilation du binaire Rust Linux | Réussies |
| Création du paquet Debian | Réussie ; détails du paquet ci-dessous |
| Démarrage du serveur de développement et requête HTTP de la page principale | Réussis ; ne valident pas la navigation desktop |
| Démarrage natif / scénario de navigation WebView | Bloqué avant exécution du scénario, sortie 133 |
| Windows, macOS, AppImage et workflow GitHub Actions | Non exécutés |
| Mesures de RAM, CPU et démarrage de l’application fonctionnelle | Non réalisées |

Le test jsdom exerce l’interface compilée dans un DOM simulé ; il ne valide ni le rendu graphique réel, ni WebKit, ni la navigation. Les tests Rust d’isolation portent sur la validation des commandes et identifiants, pas sur un audit complet des moteurs web.

## Paquet Linux généré

`src-tauri/target/release/bundle/deb/NOVA Browser_0.1.0_amd64.deb` : **3 713 078 octets** (3,54 Mio). Binaire de production : **9 462 216 octets**. Dépendances déclarées : `libwebkit2gtk-4.1-0`, `libgtk-3-0`.

L’archive a été extraite et son exécutable comparé au binaire compilé : seul le marqueur de type de paquet `UNK` → `DEB` ajouté par Tauri diffère. Cette vérification ne constitue pas un test d’installation ou d’exécution sur un poste desktop.

SHA-256 du paquet : `89023786fc9ccb3787e729dd9e43f9941f7da0c4aaf75cfd6fda3b0d3e4847d8`.

## Blocage natif constaté

L’installation `apt-get` système échoue : pas d’accès en écriture aux répertoires apt/dpkg, même après tentative d’exécution avec permissions étendues. Les paquets ont été téléchargés depuis les dépôts Debian avec vérification des métadonnées, puis extraits sous `/workspace/tooling/sysroot`. Cela permet compilation et édition de liens.

Le lancement natif échoue ensuite :

```
Unable to spawn a new child process:
Failed to spawn child process
/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess
(No such file or directory)
```

Le paquet extrait contient ce programme sous `/workspace`, mais la bibliothèque WebKitGTK distribuée attend son chemin système. La machine interdit l’installation à cet emplacement. La création d’un espace de noms utilisateur échoue également (`/proc/self/uid_map: Read-only file system`). Aucun processus WebKit n’a été exécuté avec une sandbox désactivée ; aucune bibliothèque n’a été patchée pour contourner ses protections.

Il faut une machine desktop ou un runner graphique autorisant l’installation normale de WebKitGTK et les mécanismes de sandbox pour terminer le test natif. Sous Windows, utiliser les prérequis MSVC et WebView2. Une politique réseau autorisant les sites à tester est également nécessaire.

## Portée de la livraison

Le code et le paquet sont des résultats de compilation, pas la preuve d’un navigateur prêt à l’usage quotidien. Les fonctions natives sont implémentées et le banc de test est fourni, mais leur exécution complète reste à valider. Le coût des bibliothèques WebKitGTK système n’est pas inclus dans la taille du paquet Debian. Aucune promesse de légèreté RAM/CPU n’est déduite de la taille du JavaScript ou de l’installateur.

Les champs `install_script`, `start_skill` et le chemin réel du dépôt ont été sauvegardés dans le brouillon de configuration cloud. Cette sauvegarde n’exécute pas les commandes et ne publie pas l’environnement. La restauration dans une nouvelle tâche n’a pas été testée.
