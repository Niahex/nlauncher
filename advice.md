 1. Architecture : Adopter le pattern Model-View
  Assurez-vous que la logique lourde (le scan des fichiers .desktop dans applications.rs ou la recherche DBus dans vault.rs) est encapsulée dans des `gpui::Model<T>`
  plutôt que directement dans la struct Launcher.
   - Pourquoi ? Cela permet de décorréler les données de leur représentation. Un modèle peut notifier ses observateurs (cx.notify()) dès qu'une nouvelle application est
     trouvée, permettant une mise à jour fluide de l'UI sans tout recalculer.

  2. Performance : Passer à UniformList
  Si nlauncher commence à indexer des milliers d'éléments (fichiers, entrées de coffre-fort, processus), l'utilisation d'un simple .children() dans un div() va ralentir le
  rendu.
   - Conseil : Utilisez `UniformList` (voir docs/gpui/07_LISTS_AND_OVERLAYS.md). Cela virtualise le rendu : seuls les 10 ou 20 éléments visibles à l'écran seront
     réellement calculés et envoyés au GPU.

  3. Réactivité : Annulation des recherches (Pattern Task)
  Lorsqu'un utilisateur tape vite, plusieurs recherches asynchrones peuvent se chevaucher.
   - Conseil : Utilisez le pattern de gestion de tâche avec `Task`. Stockez la tâche de recherche actuelle dans votre état. Si l'utilisateur tape un nouveau caractère,
     "droppez" l'ancienne tâche.
   - Bonus : Si vous utilisez gpui_tokio, l'annulation sera immédiate au niveau du thread (voir docs/gpui_tokio/03_TASK_MANAGEMENT.md), évitant de gaspiller du CPU sur une
     recherche dont le résultat ne sera jamais affiché.

  4. Robustesse : Utiliser SumTree pour les listes filtrées
  Si vous gérez de très grandes listes de résultats de recherche floue :
   - Conseil : Considérez l'utilisation d'un `SumTree` pour stocker les résultats triés par score. Cela permet des accès ultra-rapides ($O(\log n)$) pour récupérer les
     éléments à afficher dans la UniformList en fonction de la position du scroll.

  5. UI/UX : Thémage via les Globaux
  Au lieu de passer des couleurs partout dans vos fonctions de rendu :
   - Conseil : Définissez une struct Theme (palette Nord) et enregistrez-la comme `Global` (voir docs/gpui/14_THEMING.md).
   - Avantage : Pour implémenter un mode "Light/Dark", il vous suffira de mettre à jour l'objet global et d'appeler cx.refresh(), et toute l'interface se mettra à jour
     instantanément sans redémarrage.

  6. Intégration Linux : layer-shell et Transparence
  Puisque nlauncher est un overlay :
   - Conseil : Vérifiez que vous utilisez bien WindowBackgroundAppearance::Transparent et les options layer-shell spécifiques à Wayland pour que le lanceur n'apparaisse
     pas comme une fenêtre "normale" avec des bordures de décoration (voir docs/gpui/22_WINDOW_EFFECTS.md).

  7. Tests : Automatiser la logique de recherche
   - Conseil : Utilisez massivement `#[gpui::test]` pour tester votre moteur de recherche floue sans lancer l'interface. Simulez des saisies clavier et vérifiez que le
     premier résultat est bien celui attendu. C'est le meilleur moyen d'éviter les régressions sur la pertinence des résultats (voir docs/gpui/10_TESTING.md).
