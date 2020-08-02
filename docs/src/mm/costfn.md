# Fonction de coût

Le but des anmateurs automatisés est de determiner un prix juste en utilisant les informations des
agents sur le marché, lui meme étant non-informé.

## Règle de score de marché

Tout d'abord nous definissons une règle de score. Considerons deux variables aléatoires \\(Y: \Omega
\mapsto X\\) et \\(G: \Omega \mapsto Z\\) ou respectivement \\(Y\\) et \\(G\\) sont les probabilités
visées par l'estimation et les probabilité générés par l'estimation. Une fonction de score est
définit par \\(S: Y \times G \mapsto \mathbb{R}\\). Autrement dit il s'agit d'une fonction
mathématique associant à un ensemble de probabilités estimés un nombre que l'on peut interpreter
comme la "justesse" de l'approximation. On considère que le point qui minimise la fonction de score
est celui ou les probabilités se rapproche le de l'evennement réel.

Avec une règle de score, les agent rapportent leurs probabilités pour chaque evennement et recoivent
un paiement selon chaque réalisation. Une règle de score de marché est une règle de score ou tout le
monde peut changer ses choix à chaque instant et recevoir un paiement dependant du dernier paiement.
Le cout induit par la règle de score de marché est celui du dernier rapport comparé au premier. Dans
[Hanson (2002)](https://mason.gmu.edu/~rhanson/mktscore.pdf) l'auteur montre que toute fonction de
score rapportant des probabilités honnètes peut être utilisé comme fonction de score de marché.

## Animateurs de marché à fonction de score

Nous decrivons désormais le déroulement des échanges avec un animateur de marché automatique à
fonction de score. Ce dernier commence avec un état initiale (généralement \\(\vec{0}\\)). Les
agents interagissent avec l'animateur en modifiant son état interne de \\(x\\) à \\(x'\\) pour un
prix de \\(C(x') - C(x)\\). Par exemple pour des evennement \\(\omega_1\\) et \\(\omega_2\\) et un
état initial de \\(\\{0, 0\\}\\) si un agent souhaite acheter deux titres associés à \\(\omega_2\\)
il devra d'affranchir de \\(C(\\{0, 0 + 2\\}) - C(\\{0, 0\\})\\). Ainsi on peut qualifier le prix
d'un actif comme le gradient de la fonction de score par rapport à cet actif.

## Propriétés désirables des fonctions de score de marché

#### 1. Monotonie

\\[\forall x,y \space s.t. \space x_i \le y_i, \space C(x) \le C(y)\\]

Interprétation: le prix marginal d'un ordre ne décroit jamais, empechant d'acheter des combinaisons
à prix 0 en faisant des gains.

#### 2. Convexité

\\[\forall x,y \space and \space \lambda \in [0, 1] \\] \\[C(\lambda x + (1-\lambda) y) \le \lambda
C(x) + (1 - \lambda) C(y) \\]

Interprétation: une annonce diversifié donne un score plus faible que deux annonces séparés. Ainsi
cela incite à la diversification des portfeuilles.

#### 3. Perte borné

\\[\sup_x[\max_i(x_i) - C(x)] \lt \infty \\]

Interprétation: permet d'assurer une perte borné peu importe les actions des agents et les états
réalisés.

#### 4. Invariance à la translation

\\[\forall \vec{x},\alpha \\] \\[C(x + \vec{1} \alpha) = C(x) + \alpha \\]

Interprétation: Si on paris \\(\alpha\\) unités sur tous les états on paie \\(\alpha\\) unités.

#### 5. Homogénéité positive

\\[\forall \vec{x},\gamma \gt 0\\] \\[C(x \gamma) = C(x) \gamma \\]

Interprétation: Si on double son paris on paie double.

## Exemple de fonction de score

...
