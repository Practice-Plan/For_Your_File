# Guide de création des paquets de composants

Depuis la v0.0.5, For_Your_File organise ses fonctionnalités sous forme de **paquets de composants (Component Package)**. Un paquet de composants est un simple document JSON qui déclare la configuration d'un ensemble de fonctionnalités ; l'hôte lit et interprète cette configuration et **ne charge ni n'exécute jamais de code tiers**, l'import de paquets créés par d'autres est donc sans danger.

Ce document sert également de source au « Guide de création » intégré à l'application (la commande `get_component_creation_guide` renvoie ce fichier).

---

## 1. Structure du fichier

Un paquet de composants est simplement un fichier portant l'extension `.json` :

```json
{
  "schema": 1,
  "id": "com.example.my-pack",
  "name": "我的组件包",
  "version": "1.0.0",
  "description": "一句话说明这个组件包做什么",
  "author": "你的名字",
  "kind": "theme",
  "components": [
    {
      "id": "appearance",
      "kind": "theme",
      "title": "我的配色",
      "config": { "accent": "violet" }
    }
  ]
}
```

## 2. Champs

| Champ | Obligatoire | Description |
| --- | --- | --- |
| `schema` | Oui | Version du format, actuellement fixée à `1` |
| `id` | Oui | Identifiant unique. 3 à 64 caractères ; le premier doit être une lettre ou un chiffre, les suivants peuvent utiliser `a-z`, `0-9`, `.`, `-`, `_`. La forme `com.auteur.nom` est recommandée |
| `name` | Oui | Nom affiché, 1 à 100 caractères |
| `version` | Oui | Nombres séparés par des points, par exemple `1.0.0` (1 à 4 segments, uniquement des chiffres) |
| `description` | Non | Texte descriptif |
| `author` | Non | Informations sur l'auteur |
| `kind` | Non | Type de paquet, voir ci-dessous ; vide signifie paquet générique |
| `components` | Non | Tableau de composants, 32 au maximum |

### Objet composant

| Champ | Obligatoire | Description |
| --- | --- | --- |
| `id` | Oui | Identifiant du composant, 1 à 64 caractères |
| `kind` | Oui | Type de composant, utilisé par l'hôte pour choisir l'interpréteur |
| `title` | Oui | Titre du composant (affiché dans l'interface) |
| `config` | Non | Objet JSON libre, 256 Kio au maximum après sérialisation |

## 3. Types de paquets (kind)

| kind | Usage |
| --- | --- |
| `theme` | Apparence : couleur d'accent, taille de police, animations et autres préférences d'affichage |
| `search` | Comportement de recherche : stratégie de tri, nombre par page, option multilingue |
| `workflow` | Actions rapides : combiner des commandes existantes en boutons d'action |
| `bundle` | Paquet combiné : transporte à la fois des composants de plusieurs types |

## 4. Conventions de configuration

L'hôte lit `config` selon le `kind` du composant. Les clés actuellement reconnues sont listées ci-dessous (les clés inconnues sont ignorées et réservées aux versions futures) :

### 4.1 `theme`

```json
{ "accent": "blue", "font_scale": "normal", "reduce_motion": false }
```

| Clé | Valeurs | Description |
| --- | --- | --- |
| `accent` | `blue` `violet` `emerald` `amber` `rose` | Couleur d'accent |
| `font_scale` | `small` `normal` `large` | Taille de police de l'interface |
| `reduce_motion` | `true` / `false` | Réduire les effets d'animation |

### 4.2 `search`

```json
{ "sort": "relevance", "page_size": 50, "multilingual": true }
```

| Clé | Valeurs | Description |
| --- | --- | --- |
| `sort` | `relevance` `mostUsed` `recentlyOpened` `alphabetical` | Ordre de tri par défaut |
| `page_size` | Entier de 10 à 200 | Résultats par page |
| `multilingual` | `true` / `false` | Activer la correspondance par sous-chaîne pour les mots-clés non ASCII (CJK, cyrillique, arabe, etc.) |

### 4.3 `workflow`

```json
{
  "actions": [
    { "id": "rebuild-index", "title": "重建搜索索引", "command": "rebuild_fts_index" }
  ]
}
```

`command` doit être un nom de commande déjà enregistré par l'hôte (par exemple `open_database_preview`, `rebuild_fts_index`). Les commandes inconnues ne sont jamais exécutées ; elles sont seulement affichées désactivées.

## 5. Import, export et paquets intégrés

- **Import** : Paramètres → Paquets de composants → Importer, puis choisissez un fichier `.json`. Le schéma, l'id, la version, le kind, le nombre de composants et la taille de la configuration sont validés ; un paquet invalide est refusé avec la raison.
- **Export** : tout paquet (y compris intégré) peut être exporté en JSON. Un paquet importé peut être réimporté tel quel ; un paquet intégré exporté ne sert que de modèle — modifiez son `id` avant de le réimporter (les id intégrés sont réservés).
- **Paquets par défaut intégrés** : trois paquets font partie du binaire de l'application :
  - `fyf.appearance.classic` — apparence bleue classique
  - `fyf.search.balanced` — pertinence d'abord, 50 résultats par page, correspondance multilingue
  - `fyf.workflow.quicklaunch` — actions rapides de maintenance
  Les paquets intégrés peuvent être activés ou désactivés, mais **ne peuvent être supprimés ni écrasés** (leurs id sont réservés).
- **Emplacement** : les paquets importés sont stockés dans `components/packages/<id>.json` du répertoire de données de l'application, et l'état d'activation dans `components/state.json`.

## 6. Contraintes de sécurité

1. Les paquets de composants sont **déclaratifs** : ils ne contiennent ni script, ni exécutable, ni référence à une ressource distante, et l'hôte n'exécute jamais les chaînes d'un paquet.
2. Un fichier est limité à 1 Mio, la configuration d'un composant à 256 Kio, et un paquet à 32 composants au maximum.
3. Les id de paquet doivent être uniques ; les id intégrés ne peuvent pas être utilisés.
4. Les champs inconnus et les `kind` inconnus sont ignorés ou affichés en lecture seule, sans jamais faire planter l'application.

## 7. Exemple minimal : un thème violet personnalisé

```json
{
  "schema": 1,
  "id": "com.example.violet-theme",
  "name": "紫色夜间",
  "version": "1.0.0",
  "description": "紫色强调色 + 减弱动效",
  "author": "example",
  "kind": "theme",
  "components": [
    {
      "id": "appearance",
      "kind": "theme",
      "title": "Violet",
      "config": { "accent": "violet", "font_scale": "normal", "reduce_motion": true }
    }
  ]
}
```

Enregistrez-le sous `violet-theme.json`, puis importez-le dans l'application.
