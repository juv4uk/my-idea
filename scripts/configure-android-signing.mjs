import { readFileSync, writeFileSync } from 'node:fs';

const buildFile = 'src-tauri/gen/android/app/build.gradle.kts';
let source = readFileSync(buildFile, 'utf8');

if (source.includes('my-idea Android signing') || (source.includes('keystoreProperties') && source.includes('signingConfigs {'))) {
  console.log('Android signing already configured · Android-підпис уже налаштовано · Android-Signatur bereits konfiguriert');
  process.exit(0);
}

// The generated Android project is disposable; this deterministic patch reconnects CI secrets.
// Згенерований Android-проєкт тимчасовий; цей патч під’єднує до нього CI secrets.
// Das generierte Android-Projekt ist temporär; dieser Patch verbindet die CI-Secrets erneut.
if (!source.includes("import java.util.Properties")) {
  source = `import java.util.Properties\n\n${source}`;
}

const androidAnchor = 'android {';
const buildTypesAnchor = '    buildTypes {';
const releaseAnchor = '        getByName("release") {';

for (const anchor of [androidAnchor, buildTypesAnchor, releaseAnchor]) {
  if (!source.includes(anchor)) {
    throw new Error(`Generated Gradle anchor not found: ${anchor}`);
  }
}

source = source.replace(
  androidAnchor,
  `// my-idea Android signing · Android-підпис · Android-Signatur
val keystoreProperties = Properties().apply {
    val propertiesFile = rootProject.file("keystore.properties")
    if (propertiesFile.exists()) {
        propertiesFile.inputStream().use { load(it) }
    }
}

${androidAnchor}`,
);

source = source.replace(
  buildTypesAnchor,
  `    signingConfigs {
        create("release") {
            storeFile = file(keystoreProperties.getProperty("storeFile"))
            storePassword = keystoreProperties.getProperty("storePassword")
            keyAlias = keystoreProperties.getProperty("keyAlias")
            keyPassword = keystoreProperties.getProperty("keyPassword")
        }
    }

${buildTypesAnchor}`,
);

source = source.replace(
  releaseAnchor,
  `${releaseAnchor}
            signingConfig = signingConfigs.getByName("release")`,
);

writeFileSync(buildFile, source, 'utf8');
console.log('Android signing configured · Android-підпис налаштовано · Android-Signatur konfiguriert');
