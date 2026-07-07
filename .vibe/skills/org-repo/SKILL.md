---
name: org-repo
description: Règles d'organisation du répo pour une structure claire et minimaliste.
user-invocable: false
---

# Skill: Organisation du Répo

## Règles de Structure

1. **Spécification** : La spécification du projet **doit** être documentée dans le fichier `README.md` à la racine du répo.

2. **Dossiers Spécifiques** :
   - `schema/` : Contient **uniquement** les schémas FlatBuffers.
   - `benchmark/` : Contient **uniquement** les benchmarks de tests.
   - `justifications/` : Contient **uniquement** les documents de justification des choix techniques.

3. **Contenu de la Spécification** :
   - La spécification **ne doit pas** contenir de code source (Rust, Python, etc.).
   - Les exemples de code **doivent** être placés dans des fichiers dédiés (ex: `examples/` ou dans les justifications).
   - Les algorithmes **peuvent** être décrits en pseudo-code (ex: `ALGORITHME parse_key`).
