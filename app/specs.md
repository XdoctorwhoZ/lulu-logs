# Application Specifications

## 1. Sidebar

La toolbar horizontale est remplacée par une sidebar verticale à gauche, composée de deux zones adjacentes inspirées de VS Code.

### 1.1 Activity Bar

Barre étroite fixe (48 px) à l'extrême gauche de l'application. Elle contient uniquement des icônes disposées verticalement.

Chaque icône peut afficher un **badge** numérique (petit cercle en haut à droite) indiquant un compteur dynamique. Le badge est masqué lorsque la valeur est 0.

Les icônes sont fournies par la crate [`dioxus-free-icons`](https://crates.io/crates/dioxus-free-icons).

#### Entrées

| # | Nom        | Icône (source / nom)              | Badge                             |
|---|------------|-----------------------------------|-----------------------------------|
| 1 | Pulse      | Bootstrap / `heart-pulse`         | Nombre de sources online          |
| 2 | Sources    | Bootstrap / `folder`              | Nombre de sources connues         |
| 3 | Attributs  | Bootstrap / `tags`                | Nombre d'attributs connus         |
| 4 | Scénarios  | Bootstrap / `check2-square`       | Nombre de scénarios pending (ou total si aucun pending) |
| 5 | Contrôles  | Bootstrap / `gear`                | Aucun                             |

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
│  [�] 2  │ Pulse      │    log entries…           │
│  [🗂] 12 │ ☐ src1     │                           │
│  [🏷] 5  │ ☐ src2     │                           │
│  [☑] 3   │ …          │                           │
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

---

## 2. Lens

### 2.1 Vue d'ensemble

La **Lens** est une vue alternative à la LogList, occupant le même espace (`flex-grow`). L'utilisateur bascule entre LogList et Lens via un **sélecteur de vue** (tabs toggle) placé en haut de la zone centrale.

```
┌──────────────────────────────────────────────────┐
│ Activity │ Side Panel │ [LogList] [Lens]          │
│   Bar    │            ├───────────────────────────┤
│          │            │                           │
│          │            │   contenu de la vue       │
│          │            │   active                  │
│          │            │                           │
├──────────┴────────────┴───────────────────────────┤
│                     StatusBar                     │
└──────────────────────────────────────────────────┘
```

### 2.2 Épinglage d'un attribut

Un **clic droit** sur n'importe quelle entrée de la LogList ouvre un menu contextuel contenant l'action :

> **Épingler « {attribute} » de « {source} »** sur la Lens

- L'action est disponible pour toutes les entrées, quel que soit le type de donnée.
- Épingler le même couple `(source, attribute)` une deuxième fois est sans effet (dédoublonnage silencieux).
- Un couple épinglé peut être **désépinglé** depuis la Lens via le bouton ✕ du widget.
- L'état des pins est conservé en mémoire pendant la session ; il n'est pas persisté entre les lancements.
- Au moment de l'épinglage, le widget est **pré-rempli** avec les données historiques déjà présentes dans `state.logs` pour ce couple `(source, attribute)` (jusqu'à N valeurs, les plus récentes en premier). Les nouvelles valeurs s'y ajoutent ensuite en temps réel.

#### Épinglage combiné RX + TX

Lorsque l'entrée est de type `bytes` et que l'attribut est `"RX"` ou `"TX"`, le menu contextuel propose un second choix :

> **Épingler « RX + TX » de « {source} »** sur la Lens

- Crée un widget unique affichant les deux flux entrelacés chronologiquement.
- Les données RX sont affichées en vert (couleur terminal par défaut), les données TX en bleu (couleur d'accent).
- Le dédoublonnage s'applique sur le triplet `(source, "RX", paired="TX")`.
- Le pré-remplissage historique inclut les entrées des deux attributs, triées par timestamp.

### 2.3 Widgets

Chaque couple `(source, attribute)` épinglé est représenté par un **widget** dans la Lens. Le widget affiché dépend du type de donnée de l'attribut.

#### Types supportés et widgets associés

| Type de donnée        | Widget                                                                                      |
|-----------------------|---------------------------------------------------------------------------------------------|
| Numérique (int/float) | **Sparkline** — courbe d'évolution des N dernières valeurs reçues                           |
| Booléen               | **Timeline binaire** — barre horizontale colorée (vert/rouge) représentant l'historique des transitions vrai/faux |
| Chaîne de caractères  | **Historique texte** — liste scrollable des N dernières valeurs reçues avec timestamp        |
| Bytes                 | **Terminal sériel** — affichage des données brutes en tant que sortie de terminal ASCII avec support des séquences de contrôle, color-codage RX/TX, et auto-scroll |
| Type non supporté     | **Placeholder** — carte grisée affichant le nom du couple `(source, attribute)` et la mention « Type non pris en charge » |

> Le nombre N de valeurs conservées par widget est fixé à **1000** par défaut. Ce buffer de 1000 valeurs est partagé entre les données historiques (chargées au moment du pin) et les nouvelles données reçues en temps réel. Si l'historique contient déjà 1000 valeurs ou plus, seules les 1000 les plus récentes sont chargées ; les suivantes évincent les plus anciennes (ring buffer).
> 
> Lorsque les logs sont effacés (bouton Clear), les buffers de tous les widgets épinglés sont également vidés.

#### Anatomie d'un widget

Chaque widget possède :
- Un **en-tête** : nom de la source (muted) + `/` + nom de l'attribut (primary) + bouton ✕ (désépingler, aligné à droite)
- Un **corps** : visualisation propre au type (cf. tableau ci-dessus)
- Un **pied** : dernière valeur reçue + timestamp relatif (ex. « il y a 2 s »)

#### Terminal sériel (widget bytes)

Le widget Terminal sériel affiche les données brutes (bytes) sous forme de sortie ASCII :

- Chaque valeur reçue est décodée en UTF-8 (conversion lossy pour les séquences invalides).
- Les séquences de contrôle basiques sont traitées : `\r\n` et `\r` → saut de ligne, `\x08`/`\x7F` → suppression du dernier caractère, `\x07` (BEL) et `\x03` (Ctrl+C) → ignorés silencieusement.
- Le terminal défile automatiquement vers le bas à chaque rendu (auto-scroll).
- Apparence : fond noir (#0d0d0d), police monospace, texte vert avec glow vert (#00ff00).
- En mode combiné RX + TX, les chunks TX sont affichés en bleu (#569cd6) avec glow bleu.
- Les séquences VT100/ANSI (positionnement curseur, couleurs) ne sont **pas** supportées en V1.

### 2.4 Layouts

La Lens propose plusieurs dispositions prédéfinies, sélectionnables via un sélecteur de layout dans la barre d'en-tête de la Lens.

| Nom          | Description                                                           |
|--------------|-----------------------------------------------------------------------|
| **Colonne**  | Widgets empilés verticalement (1 par ligne), pleine largeur           |
| **Grille 2×** | 2 widgets par ligne                                                  |
| **Grille 3×** | 3 widgets par ligne                                                  |
| **Mosaïque** | Largeur automatique (min 280 px), wrapping CSS flex                   |

Le layout sélectionné est conservé en mémoire pendant la session.

### 2.5 État vide

Lorsqu'aucun attribut n'est épinglé, la Lens affiche un état vide explicite :

> *Aucun attribut épinglé. Faites un clic droit sur une entrée de la LogList pour épingler un attribut.*

### 2.6 État par défaut

Au lancement, la vue active est **LogList**. La Lens est vide (aucun pin).
