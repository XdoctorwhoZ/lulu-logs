# Application Specifications

## 1. Sidebar

La toolbar horizontale est remplacée par une sidebar verticale à gauche, composée de deux zones adjacentes inspirées de VS Code.

### 1.1 Activity Bar

Barre étroite fixe (48 px) à l'extrême gauche de l'application. Elle contient uniquement des icônes disposées verticalement.

Chaque icône peut afficher un **badge** numérique (petit cercle en haut à droite) indiquant un compteur dynamique. Le badge est masqué lorsque la valeur est 0.

#### Entrées

| # | Nom        | Icône | Badge                             |
|---|------------|-------|-----------------------------------|
| 1 | Sources    | 🗂    | Nombre de sources connues         |
| 2 | Attributs  | 🏷    | Nombre d'attributs connus         |
| 3 | Scénarios  | ☑     | Nombre de scénarios pending (ou total si aucun pending) |
| 4 | Pulse      | 💓    | Nombre de sources online          |
| 5 | Contrôles  | ⚙     | Aucun                             |

### 1.2 Side Panel

Panneau adjacent à l'Activity Bar (260 px de large), affichant le contenu contextuel de l'icône sélectionnée.

#### Comportement (toggle VS Code)

- **Clic sur une icône inactive** → ouvre le Side Panel avec le contenu correspondant ; l'icône devient active (surlignée, indicateur gauche bleu).
- **Clic sur l'icône déjà active** → ferme le Side Panel (toggle off).
- **Clic sur une autre icône** → change le contenu du Side Panel sans le fermer.

#### Header

Le Side Panel affiche en en-tête le nom de la section active (ex. « SOURCES », « ATTRIBUTS »…), en majuscules, style muted.

#### Contenu par panel

| Panel      | Contenu                                                                 |
|------------|-------------------------------------------------------------------------|
| Sources    | Champ de recherche texte, actions bulk (Tout afficher / Tout masquer), liste de checkboxes des sources connues |
| Attributs  | Idem sources, appliqué aux attributs                                    |
| Scénarios  | Liste des scénarios avec badges de statut (✅/❌/⏳), clic pour filtrer les logs |
| Pulse      | Liste des sources heartbeat avec indicateur online/offline (🟢/🔴), version, timestamp |
| Contrôles  | Boutons Pause/Resume, Auto-scroll ON/OFF, Export .lulu, Clear           |

### 1.3 Layout global

```
┌──────────────────────────────────────────────────┐
│ Activity │ Side Panel │       LogList             │
│   Bar    │ (260 px)   │    (flex-grow)            │
│ (48 px)  │            │                           │
│          │            │                           │
│  [🗂] 12 │ Sources    │    log entries…           │
│  [🏷] 5  │ ☐ src1     │                           │
│  [☑] 3   │ ☐ src2     │                           │
│  [💓] 2  │ …          │                           │
│  [⚙]    │            │                           │
├──────────┴────────────┴───────────────────────────┤
│                     StatusBar                     │
└──────────────────────────────────────────────────┘
```

- **Activity Bar** : pleine hauteur, largeur fixe 48 px.
- **Side Panel** : pleine hauteur, largeur fixe 260 px, masquable (absent du DOM quand aucun panel actif).
- **LogList** : occupe tout l'espace horizontal restant (`flex-grow`).
- **StatusBar** : pleine largeur en bas, hors du flux horizontal (layout colonne parent).

### 1.4 État par défaut

Au lancement, le Side Panel s'ouvre sur **Sources**.
