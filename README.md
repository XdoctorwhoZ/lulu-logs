# lulu-logs — Spécification du protocole de logging MQTT

> **Version** : 1.3.0
> **Date** : 2026-02-27

Lulu-Logs est un système de logging conçu pour fusionner des données de test hétérogènes dans une timeline unique et produire des rapports de test interactifs.

Ce dépôt contient la **spécification du protocole** lulu-logs. Les implémentations (application desktop, client Rust, etc.) sont hébergées dans des dépôts séparés.

---

## Table des matières

1. [Vue d'ensemble](#1-vue-densemble)
2. [Convention des topics](#2-convention-des-topics)
3. [Structure du payload (FlatBuffers)](#3-structure-du-payload-flatbuffers)
   - [3.4 Types spéciaux — spans génériques et dérivés](#34-types-spéciaux--spans-génériques-et-dérivés)
4. [Schéma FlatBuffers](#4-schéma-flatbuffers)
5. [Exemples](#5-exemples)
6. [Règles d'encodage et contraintes](#6-règles-dencodage-et-contraintes)
7. [Mécanisme de heartbeat (lulu-pulse)](#7-mécanisme-de-heartbeat-lulu-pulse)

---

## 1. Vue d'ensemble

Le protocole `lulu-logs` définit un canal de transport MQTT structuré pour les événements de logging. Toute l'information d'identification (source et attribut) est portée par le **topic MQTT**, tandis que le **payload** ne contient que les données métier, sérialisées en binaire au format [FlatBuffers](https://flatbuffers.dev/).

---

## 2. Convention des topics

### 2.1 Format général

```
lulu/{source_segment_1}/{source_segment_2}/.../{source_segment_n}/{attribute_name}
```

| Segment | Cardinalité | Description |
|---------|-------------|-------------|
| `lulu` | 1 (fixe) | Préfixe fixe du protocole |
| `{source_segment_i}` | 1..N | Segments formant le nom de la source (hiérarchie libre) |
| `{attribute_name}` | 1 (dernier niveau) | Nom de l'attribut ou métrique loggée — **toujours le dernier nœud du topic, sur un seul niveau** |

### 2.2 Règles de nommage des segments

- Les segments sont des chaînes de caractères alphanumériques en minuscules.
- Le séparateur autorisé au sein d'un segment est le tiret `-`.
- Aucun segment ne peut être vide.
- `{attribute_name}` est un identifiant simple (pas de `/` imbriqué).

### 2.3 Exemples de topics valides

| Topic | Source (multi-niveaux) | Attribut |
|-------|------------------------|----------|
| `lulu/mcp/filesystem/read-file` | `mcp/filesystem` | `read-file` |
| `lulu/mcp/github/pull-request/status` | `mcp/github/pull-request` | `status` |
| `lulu/psu/power-supply/channel-1/voltage` | `psu/power-supply/channel-1` | `voltage` |
| `lulu/my-service/heartbeat` | `my-service` | `heartbeat` |

### 2.4 Souscription globale

Pour recevoir tous les logs de toutes les sources et tous les attributs :

```
lulu/#
```

Pour recevoir tous les logs d'une source spécifique (tous attributs) :

```
lulu/mcp/filesystem/#
```

Pour recevoir uniquement un attribut précis sur toutes les sources :

```
lulu/+/+/status
```

---

## 3. Structure du payload (FlatBuffers)

Le payload MQTT est un **buffer binaire FlatBuffers** encodant une table `LogEntry`.

### 3.1 Champs de `LogEntry`

| Champ | Type FlatBuffers | Obligatoire | Description |
|-------|-----------------|-------------|-------------|
| `timestamp` | `string` | Oui | Horodatage UTC de l'événement au format ISO 8601 RFC 3339 avec précision milliseconde (ex. `2026-02-26T14:30:00.123Z`) |
| `level` | `LogLevel` (enum) | Oui | Niveau de sévérité de l'entrée |
| `type` | `string` | Oui | Type de la donnée transportée dans `data` — détermine comment interpréter les octets bruts (voir tableau 3.3) |
| `data` | `[ubyte]` | Oui | Valeur de la donnée sous forme de buffer binaire brut, dont l'interprétation dépend du champ `type` |

> **Note** : Le nom de la source (`source`) et le nom de l'attribut (`attribute_name`) ne figurent **pas** dans le payload — ils sont portés exclusivement par le topic MQTT.

### 3.2 Enum `LogLevel`

| Valeur entière | Identifiant | Sévérité |
|---------------|-------------|----------|
| `0` | `Trace` | Trace de développement fin-grain |
| `1` | `Debug` | Information de débogage |
| `2` | `Info` | Événement nominal *(valeur par défaut)* |
| `3` | `Warn` | Avertissement non-bloquant |
| `4` | `Error` | Erreur récupérable |
| `5` | `Fatal` | Erreur critique, arrêt probable |

### 3.3 Valeurs reconnues pour `type`

| Valeur de `type` | Encodage des octets de `data` |
|-----------------|-------------------------------|
| `"string"` | UTF-8 |
| `"int32"` | Entier signé 32 bits, little-endian |
| `"int64"` | Entier signé 64 bits, little-endian |
| `"float32"` | Flottant IEEE 754 simple précision, little-endian |
| `"float64"` | Flottant IEEE 754 double précision, little-endian |
| `"bool"` | 1 octet : `0x00` = false, `0x01` = true |
| `"json"` | Document JSON encodé en UTF-8 |
| `"bytes"` | Données binaires opaques, pas d'interprétation définie |
| `"net_packet"` | Données binaires opaques contenant un paquet réseau (voir §3.5) |
| `"serial_chunk"` | Données binaires opaques contenant un fragment de liaison série (voir §3.5) |
| `"span_beg"` | Document JSON encodé en UTF-8 (voir §3.4) |
| `"span_end"` | Document JSON encodé en UTF-8 (voir §3.4) |
| `"scenario_beg"` | Document JSON encodé en UTF-8 (voir §3.4) |
| `"scenario_end"` | Document JSON encodé en UTF-8 (voir §3.4) |
| `"tool_call_beg"` | Document JSON encodé en UTF-8 (voir §3.4) |
| `"tool_call_end"` | Document JSON encodé en UTF-8 (voir §3.4) |

### 3.4 Types spéciaux — spans génériques et dérivés

Les types `span_beg` et `span_end` marquent les bornes du cycle de vie d'une opération nommée. Ils fournissent un contrat générique réutilisable par des types spécialisés comme `scenario_beg` / `scenario_end` et `tool_call_beg` / `tool_call_end`.

L'encodage du champ `data` est identique au type `"json"` : octets UTF-8 d'un document JSON valide.

#### Contrat commun de corrélation

Le consommateur DOIT corréler un événement de fin avec son événement de début à l'aide du couple `(span_id, source MQTT)`.

- `span_id` est l'identifiant stable de corrélation.
- `name` est descriptif et non utilisé pour la corrélation.
- Un événement `*_end` sans `*_beg` correspondant DOIT être ignoré ou signalé comme anomalie.

#### `"span_beg"`

Publié au **début** d'un span générique.

| Champ JSON | Type | Obligatoire | Description |
|------------|------|-------------|-------------|
| `span_id` | string | Oui | Identifiant unique de corrélation du span |
| `name` | string | Non | Nom lisible du span |
| `kind` | string | Oui | Nature du span générique (ex. `"scenario"`, `"tool_call"`, `"calibration"`) |
| `metadata` | object | Non | Métadonnées structurées libres |

**Exemple** :
```json
{ "span_id": "span-calibration-001", "name": "calibration-window", "kind": "calibration", "metadata": { "operator": "alice" } }
```

#### `"span_end"`

Publié à la **fin** d'un span générique.

| Champ JSON | Type | Obligatoire | Description |
|------------|------|-------------|-------------|
| `span_id` | string | Oui | Même identifiant que le `span_beg` correspondant |
| `name` | string | Non | Nom lisible du span |
| `kind` | string | Oui | Même nature que le `span_beg` correspondant |
| `success` | bool | Oui | `true` = scénario réussi, `false` = scénario échoué |
| `error` | string | Conditionnel | Requis si et seulement si `success` est `false` — description lisible de la cause de l'échec. Absent (ou `null`) en cas de succès. |
| `duration_ms` | uint64 | Non | Durée du span en millisecondes |
| `metadata` | object | Non | Métadonnées structurées libres |
| `result` | any JSON | Non | Résultat structuré de l'opération |

**Exemple (succès)** :
```json
{ "span_id": "span-calibration-001", "name": "calibration-window", "kind": "calibration", "success": true, "duration_ms": 84, "result": { "calibrated_channels": 4 } }
```

**Exemple (échec)** :
```json
{ "span_id": "span-calibration-002", "name": "calibration-window", "kind": "calibration", "success": false, "error": "Reference drift exceeded tolerance", "duration_ms": 103 }
```

#### `"scenario_beg"` et `"scenario_end"`

Ces deux types sont des dérivés spécialisés du contrat span pour les scénarios de test.

- `kind` est implicite et vaut `"scenario"`.
- `span_id` reste obligatoire et devient la clé de corrélation canonique.
- `name` contient le nom lisible du scénario.

**Exemple `scenario_beg`** :
```json
{ "span_id": "scenario-voltage-regulation-3v3", "name": "voltage-regulation-3v3", "metadata": { "target_voltage": 3.3, "tolerance_v": 0.05 } }
```

**Exemple `scenario_end`** :
```json
{ "span_id": "scenario-voltage-regulation-3v3", "name": "voltage-regulation-3v3", "success": true, "duration_ms": 24, "result": { "measured_min": 3.30, "measured_max": 3.31 } }
```

#### `"tool_call_beg"` et `"tool_call_end"`

Ces deux types sont des dérivés spécialisés du contrat span pour tracer le début et la fin d'un appel d'outil d'agent.

- `kind` est implicite et vaut `"tool_call"`.
- `name` contient en pratique le nom de l'outil.
- `metadata` peut contenir le nom de l'agent, un résumé des arguments et tout contexte utile.
- `result` peut contenir un résumé structuré du résultat de l'appel d'outil.

**Exemple `tool_call_beg`** :
```json
{ "span_id": "tool-call-read-file-001", "name": "read_file", "metadata": { "agent_name": "Copilot", "arguments": { "path": "README.md" } } }
```

**Exemple `tool_call_end`** :
```json
{ "span_id": "tool-call-read-file-001", "name": "read_file", "success": true, "duration_ms": 12, "metadata": { "agent_name": "Copilot" }, "result": { "status": "ok", "bytes_read": 2048 } }
```

#### `"step_beg"` et `"step_end"`

Ces deux types sont des dérivés spécialisés du contrat span pour tracer les étapes individuelles au sein d'un scénario de test. Ils permettent un suivi plus granulaire que `scenario_beg`/`scenario_end`.

- `kind` est implicite et vaut `"step"`.
- `span_id` reste obligatoire et devient la clé de corrélation canonique.
- `name` contient le nom lisible de l'étape.

**Exemple `step_beg`** :
```json
{ "span_id": "step-measure-voltage-001", "name": "measure-voltage", "metadata": { "channel": 1, "expected_v": 3.3 } }
```

**Exemple `step_end`** :
```json
{ "span_id": "step-measure-voltage-001", "name": "measure-voltage", "success": true, "duration_ms": 5, "result": { "measured_v": 3.31 } }
```

### 3.5 Types binaires spécialisés — `net_packet` et `serial_chunk`

Ces deux types raffinent le type générique `"bytes"` pour les cas d'usage réseau et liaison série. Le champ `data` contient des octets bruts opaques, sans en-tête ni encapsulation supplémentaire ajoutée par le protocole `lulu-logs`.

#### `"net_packet"`

Données binaires opaques contenant un **paquet réseau** complet (ex. trame Ethernet, paquet IP, paquet UDP/TCP, etc.). Le contenu exact dépend du contexte de la source ; le consommateur doit connaître a priori le type de paquet attendu pour l'interpréter.

| Champ | Valeur |
|-------|--------|
| Encodage | Octets bruts — pas de transformation |
| Taille minimale | 1 octet |
| Taille maximale | Limitée par `MAX_PAYLOAD_SIZE` (20 480 octets) |

**Exemple d'usage** :
```
type  : "net_packet"
data  : <octets bruts d'une trame Ethernet capturée>
```

#### `"serial_chunk"`

Données binaires opaques contenant un **fragment de liaison série** (ex. octets reçus/émis sur UART, RS-232, RS-485, SPI, I²C, etc.). Peut contenir n'importe quelle séquence d'octets, y compris des octets nuls ou des caractères de contrôle.

| Champ | Valeur |
|-------|--------|
| Encodage | Octets bruts — pas de transformation |
| Taille minimale | 1 octet |
| Taille maximale | Limitée par `MAX_PAYLOAD_SIZE` (20 480 octets) |

**Exemple d'usage** :
```
type  : "serial_chunk"
data  : <octets bruts reçus sur UART à 115200 bauds>
```

---

## 4. Schéma FlatBuffers

Les fichiers de schéma FlatBuffers se trouvent dans le répertoire [`schema/`](schema/).

**Fichier** : `lulu_logs.fbs`

```fbs
// lulu_logs.fbs
// Protocol: lulu-logs v1.3.0
// Description: Schema for MQTT log payloads — source and attribute are carried
//              exclusively by the topic, not present in this payload.

namespace LuluLogs;

// ---------------------------------------------------------------------------
// Log severity levels — ordered from least to most severe.
// ---------------------------------------------------------------------------
enum LogLevel : byte {
  Trace = 0,
  Debug = 1,
  Info  = 2,
  Warn  = 3,
  Error = 4,
  Fatal = 5
}

// ---------------------------------------------------------------------------
// Root table — one per MQTT publish.
// The owning source and attribute name are NOT stored here;
// they are derived from the MQTT topic.
// ---------------------------------------------------------------------------
table LogEntry {
  // ISO 8601 UTC timestamp with millisecond precision.
  // Example: "2026-02-26T14:30:00.123Z"
  timestamp: string (required);

  // Severity level of this log entry. Defaults to Info.
  level: LogLevel = Info;

  // Type descriptor for the data field.
  // Determines how to interpret the raw bytes in `data`.
  // Known values: "string", "int32", "int64", "float32", "float64", "bool", "json", "bytes",
  //               "net_packet", "serial_chunk",
  //               "span_beg", "span_end",
  //               "scenario_beg", "scenario_end",
  //               "tool_call_beg", "tool_call_end".
  type: string (required);

  // The actual data value as a raw binary buffer.
  // Interpretation depends on the `type` field.
  data: [ubyte] (required);
}

root_type LogEntry;
```

### 4.1 Génération du code

Utiliser le compilateur `flatc` pour générer les bindings dans le langage cible :

```bash
# Rust
flatc --rust schema/lulu_logs.fbs

# Python
flatc --python schema/lulu_logs.fbs

# TypeScript
flatc --ts schema/lulu_logs.fbs

# C++
flatc --cpp schema/lulu_logs.fbs
```

Le fichier généré expose les types `LuluLogs::LogEntry` et `LuluLogs::LogLevel`.

---

## 5. Exemples

### 5.1 Publication d'un log `Info` simple

**Topic** :
```
lulu/mcp/filesystem/read-file
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:30:00.123Z"
level     : Info
type      : "string"
data      : <octets UTF-8 de "Tool call completed successfully">
```

### 5.2 Publication d'un log `Error` avec données JSON

**Topic** :
```
lulu/mcp/github/pull-request/merge
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:31:05.456Z"
level     : Error
type      : "json"
data      : <octets UTF-8 de '{"pr_id": 42, "conflicting_files": ["src/main.rs", "Cargo.toml"], "duration_ms": 312}'>
```

### 5.3 Début d'un scénario de test

**Topic** :
```
lulu/mcp/filesystem/scenario
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:32:00.000Z"
level     : Info
type      : "scenario_beg"
data      : <octets UTF-8 de '{"span_id": "scenario-read-file-returns-content", "name": "read-file-returns-content"}'>
```

### 5.4 Fin d'un scénario de test — succès

**Topic** :
```
lulu/mcp/filesystem/scenario
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:32:01.245Z"
level     : Info
type      : "scenario_end"
data      : <octets UTF-8 de '{"span_id": "scenario-read-file-returns-content", "name": "read-file-returns-content", "success": true, "duration_ms": 124}'>
```

### 5.5 Fin d'un scénario de test — échec

**Topic** :
```
lulu/mcp/filesystem/scenario
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:32:01.978Z"
level     : Error
type      : "scenario_end"
data      : <octets UTF-8 de '{"span_id": "scenario-read-file-returns-content", "name": "read-file-returns-content", "success": false, "error": "Expected file content \"hello\" but got empty response", "duration_ms": 178}'>
```

### 5.6 Début d'un appel d'outil d'agent

**Topic** :
```
lulu/agent/copilot/tool-call
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:32:10.000Z"
level     : Info
type      : "tool_call_beg"
data      : <octets UTF-8 de '{"span_id": "tool-call-read-file-001", "name": "read_file", "metadata": {"agent_name": "Copilot", "arguments": {"path": "README.md"}}}'>
```

### 5.7 Fin d'un appel d'outil d'agent

**Topic** :
```
lulu/agent/copilot/tool-call
```

**Payload** (représentation lisible avant sérialisation FlatBuffers) :
```
timestamp : "2026-02-26T14:32:10.012Z"
level     : Info
type      : "tool_call_end"
data      : <octets UTF-8 de '{"span_id": "tool-call-read-file-001", "name": "read_file", "success": true, "duration_ms": 12, "result": {"status": "ok", "bytes_read": 2048}}'>
```

### 5.8 Décodage côté consommateur

```
Topic reçu : lulu/psu/power-supply/channel-1/voltage
                 \___________ source ___________/ \attr/
```

| Champ déduit | Valeur |
|-------------|--------|
| Source (niveaux 1..N-1) | `psu/power-supply/channel-1` |
| Attribut (dernier niveau) | `voltage` |
| Données | Décodées depuis le buffer FlatBuffers |

---

## 6. Règles d'encodage et contraintes

| Règle | Description |
|-------|-------------|
| **Encodage binaire** | Le payload est **toujours** un buffer FlatBuffers valide (pas de JSON, pas de texte brut). **Exception** : les messages `lulu-pulse/…` utilisent un payload JSON UTF-8 brut (voir §7.2). |
| **Champ `data` binaire** | Le champ `data` est un vecteur d'octets bruts (`[ubyte]`). L'encodage exact des octets est défini par le champ `type` (voir tableau 3.3). |
| **Endianness** | FlatBuffers utilise little-endian par défaut — aucune configuration requise. |
| **Timestamp** | Toujours UTC, format RFC 3339, précision milliseconde, suffixe `Z`. |
| **Niveau par défaut** | Si le champ `level` est absent ou invalide lors du décodage, la valeur `Info` (2) est utilisée. |
| **Taille maximale** | Le payload FlatBuffers ne doit pas excéder **20 480 octets** (cohérent avec la limite `max_payload_size` du broker). |
| **QoS MQTT** | `AtMostOnce` (QoS 0) — les logs sont best-effort, aucune garantie de livraison n'est requise. |
| **Retain** | `false` — les messages ne doivent pas être retenus par le broker. |
| **Validation** | Le consommateur doit vérifier que le topic comporte au moins **3 niveaux** (`lulu` + au moins 1 segment source + 1 attribut), sinon le message est ignoré. |
| **Pulse — validation** | Le consommateur doit vérifier que le topic `lulu-pulse/…` comporte au moins **2 niveaux** (`lulu-pulse` + au moins 1 segment source), sinon le message est ignoré. |

---

## 7. Mécanisme de heartbeat (lulu-pulse)

### 7.1 Format du topic

```
lulu-pulse/{source_segment_1}/{source_segment_2}/.../{source_segment_n}
```

| Segment | Cardinalité | Description |
|---------|-------------|-------------|
| `lulu-pulse` | 1 (fixe) | Préfixe fixe du canal heartbeat |
| `{source_segment_i}` | 1..N | Segments identifiant la source — mêmes règles de nommage qu'en §2.2 |

> **Note** : Contrairement au canal `lulu/…`, il n'y a **pas** de `{attribute_name}` dans ce topic. Le dernier segment est le dernier segment de la source. Le heartbeat est une propriété de la source entière.

#### Exemples de topics `lulu-pulse` valides

| Topic | Source identifiée |
|-------|-------------------|
| `lulu-pulse/mcp/filesystem` | `mcp/filesystem` |
| `lulu-pulse/mcp/github/pull-request` | `mcp/github/pull-request` |
| `lulu-pulse/psu/power-supply/channel-1` | `psu/power-supply/channel-1` |
| `lulu-pulse/my-service` | `my-service` |

### 7.2 Format du payload

Le payload est un document **JSON UTF-8 brut** (non FlatBuffers) :

| Champ JSON | Type | Obligatoire | Description |
|------------|------|-------------|-------------|
| `timestamp` | string | Oui | Horodatage UTC de l'émission au format ISO 8601 RFC 3339, précision milliseconde, suffixe `Z` |
| `version` | string | Non | Version de la source (format libre, typiquement SemVer, ex. `"1.2.3"`). Absent si la source n'expose pas de version. |

**Exemple minimal** (sans version) :
```json
{ "timestamp": "2026-02-27T10:00:00.000Z" }
```

**Exemple avec version** :
```json
{ "timestamp": "2026-02-27T10:00:00.000Z", "version": "1.2.3" }
```

> **Extensibilité** : Des champs supplémentaires PEUVENT être ajoutés dans les versions futures. Les consommateurs DOIVENT ignorer les champs inconnus.

### 7.3 Fréquence d'émission

- Le client DOIT émettre un pulse toutes les **2 secondes** (± 200 ms de tolérance).
- Le premier pulse DOIT être émis **immédiatement** au démarrage du client, avant l'expiration de la première période.
- L'émission du pulse est indépendante de l'activité de logging sur le canal `lulu/…`.

### 7.4 Détection de déconnexion

- Un consommateur qui n'a reçu aucun pulse pour une source donnée depuis **6 secondes** (soit 3× l'intervalle nominal) DOIT considérer cette source comme **déconnectée** ou **hors-ligne**.
- À la réception d'un nouveau pulse après une période de silence, la source repasse à l'état **connectée** immédiatement.
- La minuterie de détection est réinitialisée à chaque pulse reçu pour cette source.

| État | Condition |
|------|-----------|
| `online` | Un pulse a été reçu dans les 6 dernières secondes |
| `offline` | Aucun pulse reçu depuis plus de 6 secondes (ou source jamais vue) |

### 7.5 Souscription

Pour surveiller toutes les sources :

```
lulu-pulse/#
```

Pour surveiller une source précise :

```
lulu-pulse/mcp/filesystem
```

La source est déduite de la **totalité des segments** après le préfixe `lulu-pulse/`, rejoints par `/`.

### 7.6 Contraintes QoS et retain

| Règle | Valeur |
|-------|--------|
| **QoS** | `AtMostOnce` (QoS 0) — même politique que le canal `lulu/…` |
| **Retain** | `false` — les pulses ne doivent pas être retenus par le broker |

---

## Licence

Ce projet est sous licence Apache 2.0 — voir le fichier [LICENSE](LICENSE) pour les détails.
