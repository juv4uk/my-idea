# Android releases · Android-релізи · Android-Releases

## English

Every stable Android build must use the same long-lived signing key. Losing the key or its passwords makes future APK updates incompatible with installed copies. Never commit the keystore or passwords.

The release workflow expects four GitHub Actions secrets:

- `ANDROID_KEYSTORE_BASE64` — the release JKS file encoded as one Base64 string;
- `ANDROID_KEYSTORE_PASSWORD` — keystore password;
- `ANDROID_KEY_ALIAS` — release key alias;
- `ANDROID_KEY_PASSWORD` — private-key password.

Run `powershell -ExecutionPolicy Bypass -File scripts/setup-android-signing.ps1` once from the repository root. The script prompts for passwords without echoing them, creates `private/android/my-idea-release.jks`, and uploads the four secrets. It refuses to overwrite an existing key.

When all four secrets exist, every semantic-version tag builds a signed universal APK for direct testing and an AAB for a future Google Play upload. Without them, the Android job reports the signing gate and safely skips mobile artifacts without blocking desktop releases. Keep at least two encrypted backups of the JKS file and passwords in separate locations.

The first Android milestone provides an installable application and the Language Lab. Opening an arbitrary workspace folder is temporarily disabled on Android until the native bridge supports Android's Storage Access Framework; desktop workspace access is unaffected.

## Українська

Кожна стабільна Android-збірка повинна використовувати той самий довготривалий ключ підпису. Втрата ключа або паролів зробить майбутні APK несумісними з уже встановленими копіями. Ніколи не додавайте keystore або паролі до Git.

Release workflow очікує чотири GitHub Actions secrets:

- `ANDROID_KEYSTORE_BASE64` — release JKS-файл як один рядок Base64;
- `ANDROID_KEYSTORE_PASSWORD` — пароль сховища;
- `ANDROID_KEY_ALIAS` — псевдонім release-ключа;
- `ANDROID_KEY_PASSWORD` — пароль приватного ключа.

Один раз запустіть `powershell -ExecutionPolicy Bypass -File scripts/setup-android-signing.ps1` з кореня репозиторію. Скрипт приховано запитає паролі, створить `private/android/my-idea-release.jks` і завантажить чотири secrets. Наявний ключ він не перезаписує.

Коли всі чотири секрети наявні, кожен semver-тег збирає підписаний universal APK для прямого тестування та AAB для майбутнього Google Play. Без них Android job повідомляє про signing gate і безпечно пропускає мобільні артефакти, не блокуючи desktop-релізи. Зберігайте щонайменше дві зашифровані резервні копії JKS і паролів у різних місцях.

Перший Android-етап надає програму для встановлення та Language Lab. Відкриття довільної папки workspace на Android тимчасово вимкнене, доки нативний міст не підтримуватиме Android Storage Access Framework; доступ до workspace на desktop не змінюється.

## Deutsch

Jeder stabile Android-Build muss denselben langlebigen Signaturschlüssel verwenden. Der Verlust des Schlüssels oder seiner Passwörter macht zukünftige APK-Updates mit installierten Versionen inkompatibel. Keystore und Passwörter dürfen niemals in Git eingecheckt werden.

Der Release-Workflow erwartet vier GitHub-Actions-Secrets:

- `ANDROID_KEYSTORE_BASE64` — die Release-JKS-Datei als einzelne Base64-Zeichenkette;
- `ANDROID_KEYSTORE_PASSWORD` — Keystore-Passwort;
- `ANDROID_KEY_ALIAS` — Alias des Release-Schlüssels;
- `ANDROID_KEY_PASSWORD` — Passwort des privaten Schlüssels.

Führen Sie einmal `powershell -ExecutionPolicy Bypass -File scripts/setup-android-signing.ps1` im Repository-Stamm aus. Das Skript fragt Passwörter verdeckt ab, erstellt `private/android/my-idea-release.jks` und lädt die vier Secrets hoch. Ein vorhandener Schlüssel wird nicht überschrieben.

Sind alle vier Secrets vorhanden, erzeugt jedes Semver-Tag ein signiertes universelles APK zum direkten Testen und ein AAB für eine spätere Veröffentlichung bei Google Play. Ohne Secrets meldet der Android-Job die Signatursperre und überspringt mobile Artefakte sicher, ohne Desktop-Releases zu blockieren. Mindestens zwei verschlüsselte Sicherungskopien der JKS-Datei und Passwörter müssen getrennt aufbewahrt werden.

Der erste Android-Meilenstein liefert eine installierbare Anwendung und das Language Lab. Das Öffnen beliebiger Workspace-Ordner ist unter Android vorübergehend deaktiviert, bis die native Brücke das Android Storage Access Framework unterstützt; der Desktop-Zugriff auf Workspaces bleibt unverändert.
