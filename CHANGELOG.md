# Changelog

Toutes les modifications notables de la spécification lulu-logs seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet adhère à [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Non publié] - 2026-07-03

### Ajouté

- **Nouveau format unifié v2.0.0** avec fusion de `LogEntry` et `LogRecord` en une seule structure
- **Format streamable** avec préfixe de taille pour tous les transports
- **Support multi-transport** : MQTT (optionnel), TCP, WebSocket, UDP, fichiers, mémoire
- **Nouveaux schémas FlatBuffers** :
  - `schema/lulu_logs_v2.fbs` - Schéma principal unifié
  - `schema/lulu_export_v2.fbs` - Schéma d'export simplifié
- **Documentation complète** :
  - `SPÉCIFICATION_V2.md` - Spécification complète v2.0.0
  - `MIGRATION_GUIDE.md` - Guide de migration détaillé
  - `justifications/why_streamable_unified.md` - Justification des choix techniques
- **Exemple d'implémentation** dans `examples/v2_streamable/`

### Changé

- **Architecture simplifiée** : Une seule structure `LogRecord` au lieu de deux (`LogEntry` + `LogRecord`)
- **Format de fichier** : Passage d'un tableau de records à un format streamable avec préfixe de taille
- **Encodage** : Le topic est maintenant stocké directement dans `LogRecord` (sans préfixe "lulu/")
- **file_identifier** : Changé de `"LULU"` à `"LUL2"` pour la détection de version

### Supprimé

- **Rien pour l'instant** - La compatibilité v1.4.0 est maintenue pendant la transition

### Déprécié

- **Format v1.4.0** - Toujours supporté mais déprécié au profit de v2.0.0
- **Dépendance MQTT** - MQTT devient optionnel, d'autres transports sont maintenant supportés

---

## [1.4.0] - 2026-02-26

### Ajouté

- Support des types span spécialisés : `ScenarioBeg`, `ScenarioEnd`, `StepBeg`, `StepEnd`
- Documentation complète des types span avec contrats JSON
- Exemples détaillés pour tous les types de données
- Contraintes de validation formelles
- Limite de taille maximale pour les payloads (20 480 octets)

### Changé

- **DataType** : Changé de `u8` à `u32` pour supporter plus de types
- **Amélioration de la documentation** : Ajout de sections détaillées sur les spans
- **Clarification des règles** : Précision sur l'encodage, l'endianness, etc.

---

## [1.3.0] - 2025-11-15

### Ajouté

- Types binaires spécialisés : `NetPacket` et `SerialChunk`
- Support des données opaques pour les cas d'usage réseau et série
- Documentation des cas d'usage spécialisés

---

## [1.2.0] - 2025-08-01

### Ajouté

- Support des types JSON pour les données structurées
- Type `Bytes` pour les données binaires opaques
- Amélioration de la documentation des types de données

### Changé

- **Encodage des booléens** : Standardisé sur 1 octet (`0x00` = false, `0x01` = true)

---

## [1.1.0] - 2025-03-10

### Ajouté

- Support des types flottants : `Float32` et `Float64`
- Support des entiers 64 bits : `Int64`
- Documentation des encodages binaires

---

## [1.0.0] - 2025-01-01

### Ajouté

- **Première version stable** du protocole lulu-logs
- Format de base avec `LogEntry` (timestamp, level, type, data)
- Convention des topics MQTT avec préfixe `lulu/`
- Schéma FlatBuffers initial : `lulu_logs.fbs`
- Format de fichier d'export : `lulu_export.fbs`
- Support des types primitifs : String, Int32, Bool
- Niveaux de log : Trace, Debug, Info, Warn, Error, Fatal

---

## Roadmap

### v2.1.0 (Prévu)

- **Compression optionnelle** : Support de zstd/gzip pour les données volumineuses
- **Batching** : Support de l'envoi de batches de LogRecord
- **Metadata étendue** : Ajout de champs optionnels (process_id, thread_id, etc.)
- **Négociation de version** : Mécanisme de détection de version du protocole

### v2.2.0 (Prévu)

- **Schémas évolutifs** : Support des schémas avec backward/forward compatibility
- **Indexation** : Support d'index pour les fichiers .lulu
- **Recherche** : Capacités de recherche dans les fichiers de log

### v3.0.0 (Futur)

- **Nouveau format binaire** : Évaluation de Cap'n Proto ou autre format
- **Typage fort** : Support de schémas personnalisés par source
- **Sécurité** : Signature et chiffrement des logs

---

## Notes de migration

### Migration vers v2.0.0

Voir [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) pour un guide complet.

**Résumé des étapes :**
1. Lire la [SPÉCIFICATION_V2.md](SPÉCIFICATION_V2.md)
2. Générer le code FlatBuffers v2 avec `flatc`
3. Implémenter le nouveau format à côté de l'ancien
4. Convertir les fichiers .lulu existants
5. Migrer progressivement les producteurs et consommateurs
6. Arrêter l'ancien format une fois la migration terminée

### Compatibilité

- **v2.0.0** est **compatible** avec v1.4.0 via des outils de conversion
- **MQTT** reste supporté comme transport optionnel
- **Les fichiers .lulu v1** peuvent être convertis vers v2 avec l'outil `lulu-convert`

---

## Format des entrées

Chaque entrée de changelog suit ce format :

```markdown
## [version] - date

### Ajouté
- Nouvelle fonctionnalité 1
- Nouvelle fonctionnalité 2

### Changé
- Modification 1
- Modification 2

### Supprimé
- Fonctionnalité supprimée 1

### Déprécié
- Fonctionnalité dépréciée 1

### Corrigé
- Bug corrigé 1

### Sécurité
- Vulnérabilité corrigée 1
```

---

*Ce changelog a été créé pour la branche `vibe/streamable-format-7c93b4` — 2026-07-03*
