# Feature Specification: YouTube to Everything - Core Downloader & Precision Snipper

**Feature Branch**: `001-downloader-snipper`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "Un téléchargeur vidéo simple, cool et réactif pour YouTube to Everything, avec métadonnées automatiques, snipper temporel pour ne télécharger que des segments choisis, presets intelligents (qualité maximale, compression Discord <25Mo, audio podcast, GIF) propulsé par Rune."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Téléchargement Rapide avec Détection Automatique (Priority: P1)

En tant qu'utilisateur, je colle une URL YouTube (ou elle est détectée directement depuis mon presse-papier) et je vois immédiatement le titre, l'auteur, la durée et la miniature de la vidéo sans aucun délai de chargement visible. Je clique sur "Télécharger" et le média est récupéré dans sa qualité maximale avec une jauge de progression fluide.

**Why this priority**: C'est le flux fondamental sans lequel l'application n'a aucune utilité. L'expérience de base doit être instantanée et sans friction.

**Independent Test**: Peut être testé isolément en collant une URL valide de vidéo YouTube et en vérifiant la création d'un fichier MP4 lisible dans le dossier de sortie, accompagné de sa jauge de progression en temps réel.

**Acceptance Scenarios**:

1. **Given** une application ouverte et une URL YouTube dans le presse-papier, **When** l'utilisateur clique sur le champ ou le bouton coller, **Then** les métadonnées (titre, durée, auteur, miniature) s'affichent en moins de 2 secondes.
2. **Given** une vidéo sélectionnée, **When** l'utilisateur clique sur "Télécharger" en mode qualité maximale, **Then** la progression s'affiche en pourcentage continu et le fichier est disponible à la fin dans le dossier de destination.
3. **Given** une URL invalide ou une vidéo privée/supprimée, **When** l'utilisateur tente de charger les métadonnées, **Then** un message d'erreur clair et non bloquant est affiché.

---

### User Story 2 - Découpage Temporel sans Téléchargement Complet (Snipper) (Priority: P2)

En tant qu'utilisateur souhaitant isoler un passage de 30 secondes dans une vidéo ou un live de 2 heures, je définis un point de départ (ex. 01:14:00) et un point de fin (ex. 01:14:30). Le système télécharge uniquement ce segment précis sans jamais télécharger l'intégralité du flux vidéo, économisant temps et bande passante.

**Why this priority**: C'est le principal différenciateur vis-à-vis des téléchargeurs classiques : l'accès instantané à des fragments sans devoir télécharger des gigaoctets superflus.

**Independent Test**: Tester avec une vidéo longue (> 1 heure) en demandant un segment de 30 secondes. Vérifier que la durée du téléchargement est de quelques secondes seulement et que le fichier final dure exactement 30 secondes.

**Acceptance Scenarios**:

1. **Given** une vidéo inspectée, **When** l'utilisateur active le mode "Snipper" et ajuste les bornes temporelles de début et de fin, **Then** l'intervalle valide et la durée de l'extrait résultant s'affichent en clair.
2. **Given** un extrait temporel configuré, **When** l'utilisateur lance le téléchargement, **Then** seul le segment temporel spécifié est extrait et écrit sur disque.
3. **Given** des bornes temporelles incohérentes (début supérieur à fin, ou fin dépassant la durée de la vidéo), **When** l'utilisateur tente de valider, **Then** l'interface empêche le lancement et met en surbrillance l'incohérence.

---

### User Story 3 - Presets Intelligents de Format (Priority: P3)

En tant qu'utilisateur ayant des besoins variés (partager un extrait sur Discord, écouter un podcast dans les transports, ou faire un mème), je peux sélectionner en un clic un preset cible :
- **Discord / Partage :** compression automatique sous le seuil strict de 25 Mo.
- **Audio Podcast :** extraction du son en MP3 avec jaquette et métadonnées.
- **Mème / GIF :** génération d'une boucle courte et légère.

**Why this priority**: Répond directement à la promesse "To Everything" sans obliger l'utilisateur à régler manuellement des paramètres de codecs obscurs.

**Independent Test**: Tester chaque preset avec une vidéo test et vérifier : taille < 25 Mo pour Discord, fichier `.mp3` valide pour Audio, fichier `.gif` fonctionnel pour GIF.

**Acceptance Scenarios**:

1. **Given** un extrait sélectionné, **When** l'utilisateur choisit le preset "Discord (<25 Mo)", **Then** le fichier généré ne dépasse jamais 24.5 Mo tout en conservant une lisibilité maximale.
2. **Given** une vidéo sélectionnée, **When** l'utilisateur choisit "Audio Podcast", **Then** un fichier audio léger est extrait avec le titre et l'auteur correctement renseignés.
3. **Given** un segment court sélectionné (< 10 secondes), **When** l'utilisateur choisit "GIF", **Then** un fichier image animée sans son est généré.

---

### Edge Cases

- Que se passe-t-il si la connexion réseau est interrompue au milieu d'un téléchargement ? Le système doit signaler l'interruption, nettoyer les fichiers partiels corrompus et permettre une reprise propre.
- Comment le système réagit-il si l'espace disque disponible est insuffisant ? Une notification d'erreur doit prévenir l'utilisateur avant le blocage du système.
- Que se passe-t-il sur des flux YouTube diffusés en direct (live streams) en cours ? Le système doit détecter le direct et proposer soit l'enregistrement depuis le début soit le rejet propre avec explication.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Le système DOIT extraire et présenter les métadonnées (titre, créateur, durée en secondes, URL miniature) à partir de toute URL YouTube standard ou raccourcie (`youtu.be`).
- **FR-002**: Le système DOIT permettre la saisie manuelle de l'URL ainsi que la détection assistée via le presse-papier.
- **FR-003**: Le système DOIT supporter le téléchargement ciblé d'intervalles temporels arbitraires `[T_start, T_end]` sans exiger le téléchargement préalable du flux complet.
- **FR-004**: Le système DOIT fournir au minimum 4 modes/presets d'exportation : Qualité Maximale, Partage (< 25 Mo), Audio Seul (MP3), et Image Animée (GIF).
- **FR-005**: Le système DOIT émettre un flux continu de notifications de progression indiquant le pourcentage achevé, la vitesse moyenne et l'estimation du temps restant.
- **FR-006**: Le système DOIT enregistrer les fichiers finaux dans un répertoire de destination paramétrable avec un nom de fichier nettoyé et sécurisé.
- **FR-007**: Le système DOIT fonctionner de manière asynchrone sans jamais geler l'interface utilisateur pendant les transferts ou transcodages.

### Key Entities

- **VideoMetadata**: Représente la vidéo distante inspectée (identifiant, titre, auteur, durée totale, liste des flux/résolutions disponibles, URL de miniature).
- **DownloadTask**: Représente une opération planifiée (URL cible, intervalle temporel optionnel, preset choisi, chemin de sortie).
- **ProgressUpdate**: Instantané de l'avancement d'une tâche (pourcentage 0.0 à 100.0, vitesse en octets/s, temps restant en secondes, étape courante).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: L'affichage des métadonnées d'une vidéo YouTube s'effectue en moins de 3 secondes sur une connexion réseau standard.
- **SC-002**: L'extraction d'un extrait de 30 secondes sur une vidéo de plus de 60 minutes prend moins de 10 secondes au total.
- **SC-003**: 100% des fichiers produits avec le preset "Discord" pèsent strictement moins de 24.5 Mo.
- **SC-004**: L'interface utilisateur reste réactive et fluide à 60 FPS minimum pendant l'intégralité d'un téléchargement.

## Assumptions

- L'utilisateur dispose des dépendances système nécessaires (`yt-dlp` et `ffmpeg`) accessibles sur la machine.
- L'application est prioritairement conçue pour un environnement de bureau (Linux, macOS, Windows).
- Les formats de sortie par défaut sont le conteneur MP4 pour la vidéo et MP3 pour l'audio pour une compatibilité universelle avec les lecteurs multimédias.
