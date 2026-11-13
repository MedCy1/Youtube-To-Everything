# YouTube to Everything

Télécharger une vidéo YouTube, n'en garder que le passage qui compte, et l'exporter dans le format dont on a besoin. Une vitrine du framework GPU [Rune](https://github.com/anomalyco/rune).

![Écran d'accueil](docs/screenshots/accueil.jpg)

![Studio d'export](docs/screenshots/studio.jpg)

## Ce que ça fait

- **Lecture instantanée** : titre, auteur, durée et miniature dès que le lien est collé (ou détecté dans le presse-papier).
- **Découpe sans tout télécharger** : on glisse sur la forme d'onde pour choisir le passage, et seul ce segment est récupéré.
- **Quatre exports** : MP4 qualité maximale, audio MP3, vidéo compressée pour Discord (moins de 25 Mo) et GIF. La taille estimée est affichée ; un gros GIF reste possible, l'interface propose simplement un extrait plus court.
- **Une interface qui prend les couleurs de la vidéo** : l'accent est tiré de la miniature, le décor est la miniature elle-même.

## Lancer

Prérequis : Rust et `curl`. `yt-dlp` et `ffmpeg` sont utilisés s'ils sont dans le `PATH` ; sinon l'accueil propose de les installer (Linux x86_64 et aarch64 : environ 190 Mo à télécharger, 390 Mo sur disque, dans `~/.local/share/y2e/bin`, chaque fichier étant vérifié par sa somme de contrôle). Au lancement, y2e cherche en arrière-plan une version plus récente de `yt-dlp` et propose de la mettre à jour (si elle vient de votre gestionnaire de paquets, y2e l'annonce sans y toucher : c'est à vous de la mettre à jour) ; la copie de `ffmpeg` qu'il a installée est proposée au renouvellement après 4 mois.

```bash
cargo run --release -p y2e-gui
```

Les fichiers exportés arrivent dans le dossier Téléchargements.

### Au clavier

| Touche | Action |
| --- | --- |
| Entrée (dans le champ) | Lire la source |
| Tab | Passer d'un contrôle à l'autre |
| Entrée (ailleurs) | Exporter |
| ← → | Déplacer la fin du passage de 1 s (↑ ↓ : 10 s) |
| Maj + flèches | Déplacer le début au lieu de la fin |
| Origine | Reprendre toute la vidéo |

Les flèches agissent dès qu'une source est chargée, sans avoir à sélectionner la timeline (sauf pendant la saisie dans le champ du lien).

## Organisation

```
crates/
├── y2e-core/   moteur : yt-dlp, ffmpeg, presets, progression (aucune dépendance graphique)
└── y2e-gui/    interface Rune : vue, effets GPU, couleurs
specs/          spécifications des fonctionnalités
```

Le moteur et l'interface ne communiquent que par des événements typés (`CoreEvent`), ce qui permet d'ajouter d'autres interfaces sans toucher au moteur.

## Développement

```bash
cargo test --all
cargo clippy --all -- -D warnings
cargo fmt --check
```
