# Zu Pop!_OS beitragen

## Das richtige Repository finden
Bevor Sie eine Änderung vornehmen, müssen Sie das entsprechende Repository finden. Lesen Sie den Abschnitt "Entwicklerressourcen" für Hilfe beim Auffinden des richtigen Repos.

## Ein Issue erstellen
Bei größeren Features wird empfohlen, zunächst ein Issue zur Diskussion zu erstellen, falls noch keines existiert. Die Chance, dass Ihre Arbeit gemergt wird, ist am höchsten, wenn die Diskussion im Voraus einen Konsens darüber erreicht, wie (oder ob) das Feature implementiert werden soll.

## Einen Pull Request erstellen
Forken Sie das Repository, nehmen Sie Ihre Änderungen vor und erstellen Sie dann einen Pull Request! Es hilft, ausführlich zu beschreiben, was Ihre Änderung bewirkt.

Jeder PR zu Pop!_OS-Komponenten erfordert die Genehmigung des Engineering-Teams (für Codequalität und Architektur) sowie des Qualitätssicherungs-Teams (für Stabilität und UX). Fordern Sie von beiden Teams eine Überprüfung an, damit Ihr PR wahrgenommen wird. Änderungen, die die Benutzererfahrung erheblich beeinflussen (z.B. neue GUI-Funktionen), können auch die Genehmigung des UX-Teams erfordern.

## Veröffentlichungsprozess nach dem Merge
Der Pop!_OS CI-Server baut automatisch alle 15 Minuten den master- (oder main-) Branch jedes Git-Repositorys. Alle Pakete aus diesen Git-Branches werden im [Master-Staging-apt-Repository](http://apt-origin.pop-os.org/staging/master/) veröffentlicht.

Pakete werden dann aus dem Master-Staging als reguläre Updates über PRs an das [repo-release-Repository](https://github.com/pop-os/repo-release/) freigegeben, das eine [Liste](https://github.com/pop-os/repo-release/blob/master/sync) mit Name und Version jedes aktuell veröffentlichten Pakets enthält. Nach der Aktualisierung der Liste veröffentlicht ein weiterer CI-Job automatisch die in der Liste enthaltenen Paketversionen.

### Pop!_OS Veröffentlichungshäufigkeit
Pop!_OS-Komponentenupdates wie Sicherheits-Patches, Bugfixes und sogar einige neue Features werden regelmäßig veröffentlicht (nach dem Rolling-Release-Prinzip).

Feature-Updates für von Ubuntu übernommene Pakete sowie sehr große UX-Änderungen (wie die Einführung der COSMIC-Oberfläche) werden als größere Versions-Upgrades veröffentlicht.
