Pop!\_OS
Pop!\_OS wurde für Menschen entwickelt, die ihren Computer zum Erschaffen nutzen — sei es komplexe professionelle Software und Produkte, anspruchsvolle 3D-Modelle, Informatik in der Wissenschaft oder Maker, die an ihrer neuesten Erfindung arbeiten. Die Pop!-Benutzeroberfläche hält sich im Hintergrund und bietet gleichzeitig umfangreiche Anpassungsmöglichkeiten zur Optimierung Ihres Arbeitsablaufs. Aufgebaut auf Ubuntu haben Sie Zugang zu umfangreichen Repositories mit Open-Source-Software und Entwicklungswerkzeugen.

Die erste Version von Pop!\_OS erschien am 19. Oktober 2017. Weitere Informationen finden Sie auf der [Pop!\_OS-Website](https://system76.com/pop) und in der [Pop!\_OS-Dokumentation](https://support.system76.com/).

Zweck
Dieses Repository dient der einfachen Verwaltung aller Pop!\_OS-bezogenen Quellcodes und Assets. Eine Liste aller enthaltenen Repositories finden Sie in [REPOS.md](./REPOS.md).

Binärpakete werden [in den Pop!\_OS APT-Repositories](https://apt.pop-os.org/) gehostet. Viele Pakete haben ihren Quellcode auf GitHub unter der [Pop!\_OS-Organisation](https://github.com/pop-os). Einige verwandte Komponenten oder Dokumentationen können unter der [System76-Organisation](https://github.com/system76) gehostet sein.

Entwicklerressourcen
Anweisungen zum Erstellen der Shell:
* [COSMIC (GNOME-basiert)](https://github.com/pop-os/cosmic)
* [COSMIC Epoch (Rust-basiert)](https://github.com/pop-os/cosmic-epoch)

Entwickler-Chat: https://chat.pop-os.org/pop-os/channels/development

Zu Pop!\_OS beitragen
Anweisungen und Richtlinien für Änderungen an Pop!\_OS finden Sie in [CONTRIBUTING.md](./CONTRIBUTING.md).

Abhängigkeiten
Zur Nutzung dieses Repositorys müssen folgende Pakete installiert sein:
- `python3-launchpadlib`

Skripte
Dieses Repository enthält folgende Befehle:
- `scripts/clone` - Quellcode klonen
- `scripts/debversion` - Version des Debian-Pakets anzeigen
- `scripts/ignore` - `.gitignore` generieren
- `scripts/issues` - Issues anzeigen
- `scripts/launchpad` - PPA-Pakete anzeigen
- `scripts/prs` - Pull Requests anzeigen
- `scripts/pull` - Quellcode aktualisieren
- `scripts/readme` - `REPOS.md` generieren
- `scripts/validate` - Pop!\_OS-Quellcode auf Vorhandensein von `LICENSE`-, `README`- und `TESTING`-Dokumentation prüfen
