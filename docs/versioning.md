# Versioning and inherited history · Версіонування та успадкована історія · Versionierung und übernommene Historie

## English

`my-idea` was created from `my-ide` and deliberately retains the inherited Git history. That history records how the code and ideas evolved, but it also contains commits and tags whose names describe earlier project stages. They are historical provenance, not a statement that those older products are part of the current IDE.

Release tags are immutable. We do not delete, move, or overwrite a tag after it has been created. If an intended version such as `v0.3.0` is already occupied by an inherited tag, development continues with the next free patch number. Consequently, the first release of the independent Rust language core is `v0.3.2`.

Application versions and the independent language crate have separate lifecycles. The IDE currently uses the `0.3.x` series, while `crates/my-idea-language` starts at its own library version `0.1.0`.

## Українська

`my-idea` створено на основі `my-ide` зі свідомим збереженням успадкованої Git-історії. Вона показує розвиток коду та ідей, але також містить коміти й теги, назви яких описують попередні етапи проєкту. Це історичне походження коду, а не твердження, що старі продукти входять до складу поточної IDE.

Релізні теги незмінні. Після створення ми не видаляємо, не пересуваємо й не перезаписуємо тег. Якщо запланована версія, наприклад `v0.3.0`, уже зайнята успадкованим тегом, розробка продовжується з наступного вільного patch-номера. Тому перший реліз незалежного Rust-ядра мови має версію `v0.3.2`.

Версії програми та незалежного мовного крейта мають окремі життєві цикли. IDE зараз використовує серію `0.3.x`, тоді як `crates/my-idea-language` починає власне версіонування бібліотеки з `0.1.0`.

## Deutsch

`my-idea` wurde aus `my-ide` entwickelt und bewahrt die übernommene Git-Historie bewusst. Sie dokumentiert die Entwicklung von Code und Ideen, enthält aber auch Commits und Tags, deren Namen frühere Projektphasen beschreiben. Dies ist die historische Herkunft des Codes und bedeutet nicht, dass diese älteren Produkte Bestandteil der heutigen IDE sind.

Release-Tags sind unveränderlich. Nach ihrer Erstellung werden sie weder gelöscht noch verschoben oder überschrieben. Ist eine geplante Version wie `v0.3.0` bereits durch ein übernommenes Tag belegt, wird mit der nächsten freien Patch-Nummer fortgefahren. Daher trägt das erste Release des unabhängigen Rust-Sprachkerns die Version `v0.3.2`.

Die Anwendung und das unabhängige Sprach-Crate besitzen getrennte Versionszyklen. Die IDE verwendet derzeit die Reihe `0.3.x`, während `crates/my-idea-language` mit der eigenen Bibliotheksversion `0.1.0` beginnt.
