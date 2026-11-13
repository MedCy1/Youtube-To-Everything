# Feature Specification: Interface centrée sur l'artwork

**Feature Branch**: `002-artwork-first-interface`

**Created**: 2026-11-13

**Status**: Implemented

**Input**: Refonte de `y2e-gui` pour en faire une vitrine du framework Rune : l'image de la vidéo est le décor, l'interface flotte dessus.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Une interface qui prend les couleurs de la vidéo (Priority: P1)

En tant qu'utilisateur, après avoir lu une URL, la miniature de la vidéo remplit toute la fenêtre et l'interface adopte sa teinte dominante, sans que le contraste change d'une vidéo à l'autre.

**Independent Test**: Charger deux vidéos de couleurs différentes et vérifier que l'accent (timeline, bouton d'export) change de teinte tout en restant lisible.

**Acceptance Scenarios**:

1. **Given** une source chargée avec sa miniature, **When** l'image est décodée, **Then** la teinte d'accent est dérivée de la couleur dominante et la transition est animée.
2. **Given** une miniature sans couleur dominante (niveaux de gris), **When** elle est décodée, **Then** la teinte de marque est conservée.

### User Story 2 - Découper en glissant sur la timeline (Priority: P1)

En tant qu'utilisateur, je choisis le passage à exporter en glissant sur une forme d'onde. La vidéo entière reste sélectionnée tant qu'aucune poignée n'a bougé.

**Acceptance Scenarios**:

1. **Given** une source chargée, **When** je glisse près d'une borne, **Then** la poignée la plus proche suit le pointeur et la durée sélectionnée s'affiche.
2. **Given** une sélection, **When** je la ramène sur toute la vidéo ou appuie sur « Tout », **Then** le découpage est désactivé.
3. **Given** le pointeur au-dessus de la timeline, **Then** une ligne et l'heure correspondante le suivent sans reconstruire la vue.

### User Story 3 - Une fin d'export lisible (Priority: P2)

**Acceptance Scenarios**:

1. **Given** un export terminé, **Then** un halo quitte le bouton, un bandeau indique le chemin du fichier et propose d'ouvrir son dossier.
2. **Given** un export en échec, **Then** un bandeau d'erreur s'affiche sans bloquer l'interface.

### User Story 4 - Clavier et presse-papier (Priority: P2)

**Acceptance Scenarios**:

1. **Given** un lien YouTube copié dans le presse-papier, **When** la fenêtre passe au premier plan sur l'écran d'accueil, **Then** le champ est rempli et l'interface invite à appuyer sur Entrée.
2. **Given** le champ du lien, **When** j'appuie sur Entrée, **Then** la source est lue.
3. **Given** une source chargée, **When** j'appuie sur les flèches (hors saisie dans le champ), **Then** la fin du passage bouge de 1 s (10 s avec ↑ ↓), le début avec Maj, sans avoir à sélectionner la timeline.
4. **Given** une source chargée, **When** je navigue au clavier, **Then** chaque contrôle est atteignable, nommé pour les lecteurs d'écran et affiche un anneau de focus ; Entrée lance l'export hors du champ du lien.

## Requirements *(mandatory)*

- **FR-001**: La couleur d'accent DOIT être dérivée de la teinte dominante de la miniature, à luminosité et chroma fixes.
- **FR-002**: Les mouvements d'ambiance (dérive du décor, respiration de l'image, ligne de survol) DOIVENT passer par des valeurs animées côté rendu et ne pas reconstruire la vue.
- **FR-003**: Le champ URL DOIT rester un seul élément entre l'accueil et la barre supérieure.
- **FR-004**: Le chemin du fichier final DOIT provenir de `yt-dlp` (`--print after_move:filepath`), jamais d'un parsing de son journal.
- **FR-006**: Seul un lien unique vers un domaine YouTube connu DOIT être repris depuis le presse-papier ; tout autre contenu est ignoré.
- **FR-007**: Les erreurs des outils DOIVENT être traduites en une phrase claire ; le détail technique n'apparaît que si l'erreur n'est pas reconnue.
- **FR-005**: Toute logique non graphique (extraction de teinte) DOIT être couverte par un test unitaire déterministe.

## Success Criteria *(mandatory)*

- **SC-001**: Texte et contrôles restent à au moins 14 px et lisibles sur toute miniature.
- **SC-002**: Aucun traitement lourd ne s'exécute sur le thread d'interface.

## Assumptions

- Le décodage de la miniature et le calcul de teinte s'exécutent hors du thread d'interface.
- Les formats d'export et leurs règles restent ceux de la spécification 001.
