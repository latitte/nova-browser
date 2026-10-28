# Faisabilité et architecture

## Choix du MVP

Tauri 2 + Svelte 5/TypeScript + Rust + SQLite. Le document Svelte reste dans la WebView locale `main`. Chaque page Internet occupe une **WebView enfant native** distincte (`tab-UUID`) dans la même fenêtre. Aucune iframe. Le contenu distant ne remplace jamais l’interface privilégiée.

API examinées dans le code source Tauri 2.12.5 réellement résolu : `Window::add_child`, `WebviewBuilder::on_navigation`, `on_page_load`, `on_document_title_changed`, `on_new_window`, `on_download`, `on_permission_request`, `Webview::navigate`, `set_zoom`, `clear_all_browsing_data`. Les WebViews enfants nécessitent la fonctionnalité Cargo `unstable` : il s’agit d’un risque de maintenance explicite. Cargo.lock et package-lock.json fixent les versions résolues.

Références : https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewBuilder.html et https://v2.tauri.app/start/prerequisites/ . Le comportement effectif dépend de WebView2 sous Windows, WebKitGTK sous Linux et WKWebView sous macOS.

## Compromis avec Qt WebEngine

Qt WebEngine apporte Chromium et des API de navigateur plus complètes (profils, permissions, téléchargements, recherche dans la page). Son moteur embarqué augmente la taille distribuée et impose une cadence de mises à jour de sécurité. Tauri réutilise le moteur système et évite d’embarquer Chromium sous Linux/macOS, mais n’assure pas la même compatibilité web ou les mêmes API sur toutes les plateformes. Ni le poids final multiplateforme ni les gains de RAM ne sont démontrés ici.

Tauri est retenu pour ce MVP. Si la validation Windows/Linux des profils privés, de Ctrl+F et des permissions exige une convergence impossible via ces API, la prochaine décision doit comparer un prototype Qt WebEngine avec des tests fonctionnels identiques avant de poursuivre. Ce MVP n’est pas un remplacement complet de Chrome.

## Frontières de confiance

- Chaque commande native contrôle le label de sa WebView appelante. Seul `main` peut écrire les données, créer ou manipuler les onglets.
- Les capacités Tauri sont locales, limitées à `main`, sans origine distante autorisée. Aucun plugin shell ou système de fichiers exposé aux sites.
- Les URL sont parsées en Rust et limitées à HTTP(S), sans identifiants intégrés. Les protocoles `javascript:`, `file:`, `data:` et les protocoles applicatifs sont refusés.
- Les scripts évalués pour retour/suivant/arrêt sont des constantes. Les commandes n’acceptent pas de JavaScript arbitraire.
- Les requêtes SQLite utilisent des paramètres. Migration transactionnelle versionnée, index d’historique, WAL, délai d’attente en cas de verrouillage. Les versions de schéma futures sont refusées.
- Aucun contournement TLS, sandbox ou vérification de paquet. Aucune télémétrie applicative ajoutée. Les sites et moteurs de recherche ont leurs propres pratiques.
- Permissions refusées par le callback Tauri. Cela ne constitue pas une garantie de couverture de toutes les permissions sur les trois moteurs ; audit et essais nécessaires.
- Les téléchargements sont refusés : aucun fichier issu d’un site n’est lancé ou écrit par NOVA dans ce MVP.

## Organisation

`App.svelte` orchestre les onglets et le pont IPC ; `Home.svelte` compose la page d’accueil ; `Settings.svelte` gère les contrôles de personnalisation. `core.ts` contient la résolution des adresses testée sans interface. `style.css` contient le design system (surfaces, textes, accent, espacements, états, mouvements réduits).

`src-tauri/src/main.rs` regroupe les commandes natives et la persistance. `smoke.rs` est un banc de test de développement, exclu de la version de production. La session est restaurée sans créer les WebViews inactives tant qu’elles ne sont pas sélectionnées. Un onglet déjà chargé reste vivant lorsqu’on le masque : ce n’est pas une suspension. Sa fermeture appelle `Webview::close`.

La prochaine séparation native sera un service de profils explicites, puis des modules navigation/persistance/téléchargements. Un profil privé exige un contexte réellement isolé et sa validation sur chaque système ; il n’est pas simulé par un simple booléen d’interface.

## Limites fonctionnelles connues

Pas de mode privé, gestionnaire de profils, multi-fenêtres, dossiers de favoris, gestionnaire de téléchargements, indicateur audio, suspension automatique, blocage publicitaire, recherche Ctrl+F ni moteur personnalisé. Les popups HTTP(S) sont redirigées vers un nouvel onglet sans préserver la relation `window.opener` : certains parcours d’authentification peuvent échouer. Pas de support des URL internes `about:` ou des extensions.

Les raccourcis Svelte sont disponibles lorsque l’interface locale a le focus ; ils ne sont pas encore globaux lorsqu’une page distante a le focus. Le zoom est une commande explicite du menu, pas un état propre à chaque onglet. Les erreurs réseau et certificats reposent sur l’affichage du moteur natif, sans page d’erreur NOVA personnalisée. L’effacement des données vise les WebViews ouvertes ; il ne garantit pas une purge exhaustive de tous les profils historiques. Les suggestions utilisent les 500 visites les plus récentes et les favoris. L’historique conserve le domaine comme libellé initial ; les titres des onglets sont actualisés séparément.
