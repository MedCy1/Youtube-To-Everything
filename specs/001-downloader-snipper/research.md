# Phase 0 Research: moteur y2e et intégration Rune

## 1. yt-dlp : extraction des métadonnées et streaming

### Extraction des métadonnées
La commande d'inspection :
```bash
yt-dlp --dump-json --skip-download --no-playlist "<URL>"
```
Elle renvoie en sortie standard un objet JSON contenant :
- `id`, `title`, `uploader`, `duration` (en secondes)
- `thumbnail` (URL de la meilleure miniature)
- `formats` : tableau de streams vidéo/audio
- `chapters` : liste d'objets `{ "start_time": f64, "end_time": f64, "title": String }`

### Découpe temporelle sans téléchargement complet
`yt-dlp` supporte la syntaxe `--download-sections` couplée aux requêtes HTTP Range :
```bash
yt-dlp --download-sections "*01:15:00-01:15:30" --force-keyframes-at-cuts "<URL>" -o "output.%(ext)s"
```
Le serveur YouTube ne transmet que les segments de données correspondant aux timestamps demandés, ce qui ramène le téléchargement d'un segment de 30 secondes à quelques secondes, même sur une vidéo de 3 heures.

### Suivi de la progression
`yt-dlp` supporte un format d'avancement personnalisé via `--newline` et `--progress-template` :
```bash
yt-dlp --newline --progress-template "DOWNLOAD_PROGRESS:%(progress._percent_str)s:%(progress._speed_str)s:%(progress._eta_str)s"
```
Le format est stable, donc le parsing en Rust se fait par simple découpage de texte.

---

## 2. Presets d'encodage et de transcodage FFmpeg

### Preset Discord (< 25 Mo)
Pour garantir qu'une vidéo tienne dans la limite de 25 Mo (seuil strict 24.5 Mo pour marge de sécurité) :
1. Calcul du débit binaire global :
   $$\text{Total Bitrate (kbps)} = \frac{24.5 \times 8192}{\text{Durée (secondes)}}$$
2. Allocation : Audio = 96 kbps (AAC), Vidéo = $\text{Total Bitrate} - 96$.
3. Encodage FFmpeg :
   ```bash
   ffmpeg -i input.mp4 -c:v libx264 -b:v <video_bitrate>k -maxrate <video_bitrate>k -bufsize <2*video_bitrate>k -c:a aac -b:a 96k output_discord.mp4
   ```

### Preset audio podcast
Extraction pure de la piste audio avec conversion MP3 ou M4A et incorporation de la miniature :
```bash
yt-dlp -x --audio-format mp3 --audio-quality 0 --embed-thumbnail "<URL>" -o "output.%(ext)s"
```

### Preset GIF / mème
Génération d'une palette optimisée en 2 passes en une seule ligne FFmpeg :
```bash
ffmpeg -i input.mp4 -vf "fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" output.gif
```

---

## 3. Architecture Rune et concurrence

- Modèle réactif de Rune : La fonction `view(s: &State, cx: &mut Cx<State>) -> El` reconstruit l'arbre d'éléments GPU à chaque mutation de `State`.
- Pont asynchrone : les appels `yt-dlp` et `ffmpeg` tournent dans un runtime `tokio`. Un canal `std::sync::mpsc` transmet les actions de l'interface au runtime, et le handle de l'application (`AppHandle::update`) ramène les résultats dans l'état de Rune.
