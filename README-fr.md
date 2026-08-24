# Centre de gestion des raccourcis LNK

<img src="src-tauri/icons/Square89x89Logo.png" alt="icon Logo" width="80" height="80" style="border-radius: 12px;">

> Une application de bureau moderne pour gérer les raccourcis Windows .lnk, construite avec Tauri 2.0.

[中文](README-zh.md) | [English](README.md) | Français | [Русский](README-ru.md) | [العربية](README-ar.md)

![License](https://img.shields.io/badge/License-GPL--3.0-blue.svg)
![Version](https://img.shields.io/badge/Version-0.0.3-green.svg)
![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg)
![Rust](https://img.shields.io/badge/Rust-1.77.2+-orange.svg)
![Node](https://img.shields.io/badge/Node-18%2B-339933.svg)

Le Centre de gestion des raccourcis LNK vous aide à centraliser la gestion des raccourcis Windows (.lnk). Grâce à la recherche en texte intégral, au regroupement intelligent, aux rappels d’expiration, aux raccourcis globaux et au lancement en un clic, vous pouvez enfin mettre fin au chaos des raccourcis répartis sur le bureau et dans le menu Démarrer.

## ✨ Fonctionnalités

- **🔍 Recherche en texte intégral** — Recherche FTS5 SQLite en temps réel avec correspondance de préfixe, pagination, historique de recherche et surbrillance des mots-clés
- **📁 Analyse des applications** — Scan automatique des applications du menu Démarrer, extraction d’icônes Win32 natives et cache disque (chargement ultra-rapide)
- **🚀 Lancement en un clic** — Double-cliquez pour lancer des applications, ouvrir des fichiers, dossiers ou URL, avec suivi automatique de la fréquence d’utilisation
- **🖥️ Raccourcis globaux** — Raccourcis globaux personnalisables (par défaut `Alt+Space`), détection des conflits et suggestions
- **⏰ Rappels d’expiration** — Définissez des dates d’expiration pour les fichiers temporaires, recevez des alertes et nettoyez en lot à l’échéance
- **📚 Regroupement intelligent** — 8 couleurs de regroupement, affectation par glisser-déposer, opérations par lot et import/export de groupes (JSON/CSV/HTML)
- **📦 Import en lot** — Glisser-déposer ou parcourir pour importer plusieurs éléments, avec configuration unifiée des balises, paramètres et mode d’ouverture, et barre de progression en direct
- **🌐 Internationalisation** — Prise en charge du chinois, de l’anglais, du français, du russe et de l’arabe
- **🎨 Changement de thème** — Thèmes clair/sombre avec mémorisation des préférences utilisateur
- **🖱️ Menu contextuel** — Intégration au menu contextuel de l’Explorateur Windows pour ajouter rapidement des éléments
- **🔗 Liens profonds** — Prise en charge du protocole `filemgmt://` pour add/open/search/settings
- **🧩 Intégration PPC** — Connexion au système central PPC (v0.0.7) avec correspondance des codes d’erreur
- **🖥️ Zone de notification** — Masquage dans la zone de notification lors de la fermeture, avec rappel via raccourci ou icône

## 🛠️ Stack technologique

| Couche | Technologie |
|----|------|
| Framework | Tauri 2.0 |
| Frontend | React 18 + TypeScript 5 |
| Outil de build | Vite 5 |
| Style | Tailwind CSS 3 |
| Animation | Framer Motion |
| Backend | Rust (édition 2021) |
| Base de données | SQLite (rusqlite, bundled) + recherche FTS5 |
| Internationalisation | i18next (en/zh/fr/ru/ar) |
| Extraction d’icônes | Win32 API (SHGetFileInfoW + GDI) |

## 📋 Configuration requise

- **Système d’exploitation** : Windows 10 / Windows 11
- **Runtime** : Microsoft Edge WebView2 Runtime (préinstallé sur Windows 11, requis sur Windows 10)
- **Développement** : Rust 1.77.2+, Node.js 18+

## 🚀 Démarrage rapide

### Mode développement

```bash
# 1. Installer les dépendances du frontend
npm install

# 2. Démarrer le serveur de développement (port par défaut 1420)
npm run dev
```

### Build de production

```bash
# Recommandé : construire l’application Tauri complète (frontend + ressources intégrées automatiquement)
npx tauri build

# Générer le package d’installation (64 bits)
npx tauri build --target x86_64-pc-windows-msvc

# Générer le package d’installation (32 bits ; exécuter d’abord rustup target add i686-pc-windows-msvc)
npx tauri build --target i686-pc-windows-msvc
```

Les artefacts sont générés dans `src-tauri/target/<target>/release/bundle/` et comprennent les installateurs `msi/` et `nsis/`.

> **⚠️ Important** : lancer `cargo build --release` seul produit un binaire cassé, car il tente de se connecter au serveur de développement et peut échouer avec « Connection Refused ». Utilisez toujours `npx tauri build` ou `cargo build --release --features custom-protocol` pour intégrer correctement les ressources frontend.

## 📂 Structure du projet

```text
For_Your_File/
├── src/                          # Frontend React
│   ├── components/               # Composants UI (30+)
│   │   ├── BatchImportModal.tsx  # Import groupé (barre de progression / résumé des erreurs)
│   │   ├── AppSelectorModal.tsx  # Sélecteur d’applications (cache d’icônes / contrôle de concurrence)
│   │   └── ...
│   ├── hooks/                    # Hooks personnalisés (useSearch, etc.)
│   ├── locales/                  # Traductions en 5 langues
│   ├── types/                    # Types TypeScript
│   └── App.tsx                   # Application principale
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── commands.rs           # 60+ commandes Tauri
│   │   ├── db.rs                 # Schéma SQLite + déclencheurs FTS5
│   │   ├── hotkey.rs             # Gestion des raccourcis globaux
│   │   ├── lnk.rs                # Analyse des LNK (COM/IShellLinkW)
│   │   ├── app_scanner.rs        # Analyse du menu Démarrer + extraction d’icônes natives
│   │   ├── expiration/           # Système de rappels d’expiration
│   │   ├── ppc_linker.rs         # Intégration PPC
│   │   └── ...
│   ├── tests/                    # Tests d’intégration Rust
│   └── tauri.conf.json           # Configuration Tauri
├── docs/                         # Documentation
│   ├── tech/                     # Documentation technique
│   └── user/                     # Guide utilisateur
├── .github/workflows/ci.yml      # CI (tests + packaging 32/64 bits)
├── package.json
└── LICENSE.md                    # GPL-3.0
```

## 🧪 Tests

```bash
# Vérification du type frontend
npm run type-check

# Tests unitaires et d’intégration Rust
cd src-tauri && cargo test

# Vérification Clippy
cargo clippy -- -D warnings

# Vérification du formatage
cargo fmt -- --check
```

## 🔧 Base de données

- **Emplacement** : `%APPDATA%/wang.station/app/For_Your_File/lnk_management.db`
- **Tables** : `entries` (éléments), `groups` (groupes), `entry_groups` (relation plusieurs à plusieurs), `entries_fts` (index de recherche FTS5)
- **Cache des icônes** : `%APPDATA%/wang.station/app/For_Your_File/icon_cache/` (clé de hachage + invalidation par heure de modification)

## 📄 Documentation

- [Documentation technique](docs/tech/README.md) — architecture, référence API, conception de la base de données, build et déploiement
- [Guide utilisateur](docs/user/README.md) — installation, utilisation des fonctionnalités, raccourcis, FAQ, dépannage

## 🤝 Contribution

Les contributions sont les bienvenues via les issues et les pull requests. Veuillez vous assurer que :

1. Le code passe `cargo fmt` et `cargo clippy`
2. Les tests passent (`cargo test`)
3. Les messages de commit décrivent clairement les changements

## 📜 Licence

Ce projet est open source sous la licence [GNU General Public License v3.0](LICENSE.md).

Copyright © 2026 LNK File Management Center Contributors
